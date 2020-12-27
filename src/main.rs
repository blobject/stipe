use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_files::NamedFile;
use ammonia::clean;
use regex::Regex;
use std::collections::{BTreeSet, BTreeMap};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::result::Result;
use std::sync::Mutex;


#[actix_web::main]
async fn main() ->
  io::Result<()>
{
  use actix_web::{guard, App, HttpServer};

  let state = web::Data::new(State {
    site: "b.agaric.net",
    dir: "db/",
    scope: "/page",
    pub_dir: "pub",
    pub_scope: "/pub",
    pages: Mutex::new(BTreeMap::new()),
  });
  for dir in [state.dir, state.pub_dir].iter() {
    if !Path::new(dir).is_dir() {
      return Err(io::Error::new(io::ErrorKind::NotFound,
                                [dir, " not a directory"].join("")));
    }
  }
  state.up_state("b.agaric.net", "", "")
    .unwrap_or_else(|e| println!("Err: main/{}", e));

  HttpServer::new(move || {
    App::new()
      .app_data(state.clone())
      .service(actix_files::Files::new(state.pub_scope,
                                       state.pub_dir).show_files_listing())
      .route("/favicon.ico", web::get().to(route_fav))
      .route("/", web::get().to(route_root))
      .route(&[state.scope, "s"].join(""), web::get().to(route_pages))
      .route(&[state.scope, "s/"].join(""), web::get().to(route_pages))
      .default_service(web::resource("").route(web::get().to(flip))
                       .route(web::route()
                              .guard(guard::Not(guard::Get()))
                              .to(HttpResponse::MethodNotAllowed)))
  }).bind("localhost:8000")?.run().await
}


/*** error ********************************************************************/

struct Error(String);

impl std::fmt::Display for Error
{
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
  {
    f.write_str(&self.to_string())
  }
}


/*** routing ******************************************************************/

async fn route_fav(_: HttpRequest) ->
  actix_web::Result<NamedFile>
{
  Ok(NamedFile::open("pub/favicon.ico")?)
}


async fn route_root<'state>(
  state: web::Data<State<'state>>,
  req: HttpRequest,
) -> impl Responder
{
  HttpResponse::Ok().body(
    html(/* error   */ "",
         /* site    */ state.site,
         /* host    */ req.connection_info().host(),
         /* link    */ "/",
         /* query   */ "",
         /* scripts */ &vec![],
         /* html    */ &html_pagelet(/* class */ "welcome",
                                     /* title */ "welcome",
                                     /* text  */ &root_motd(),
                                     /* more  */ "")))
}


async fn route_pages<'state>(
  state: web::Data<State<'state>>,
  req: HttpRequest,
) -> impl Responder
{
  let site = state.site;
  let conn = req.connection_info();
  let host = conn.host();
  state.up_state(host, "", "")
    .unwrap_or_else(|e| println!("Err: route_pages/{}", e));
  let mut pages = match state.pages.lock() {
    Ok(o) => o,
    Err(_) => {
      println!("Err: route_pages: pages lock");
      return respond_error(500, site, host, "pages", "", "pages");
    },
  };

  let pages_total = pages.len();
  let (query, tags_list, pages_list) =
    &pages_lists(req.query_string(), &mut pages);
  if 0 == pages_list.len() && 0 < pages_total {
    return respond_error(404, site, host, "pages", "no pages tagged", query);
  }

  HttpResponse::Ok().body(
    html(/* error   */ "",
         /* site    */ site,
         /* host    */ host,
         /* link    */ "pages",
         /* query   */ query,
         /* scripts */ &vec![],
         /* html    */ &html_pages(/* query */ query,
                                   /* total */ pages_total,
                                   /* tags  */ tags_list,
                                   /* pages */ pages_list)))
}


async fn flip<'state>(
  state: web::Data<State<'state>>,
  req: HttpRequest,
) -> impl Responder
{
  let conn = req.connection_info();
  let host = conn.host();
  let site = state.site;
  let dir = state.dir;
  let scope = state.scope;
  let (bad, msg, name, link) = &flip_lick(req.match_info().path(), dir,
                                          &[scope, "/"].join(""));
  if *bad {
    return respond_error(404, site, host, link, *msg, link);
  }

  state.up_state(host, &link, name)
    .unwrap_or_else(|e| println!("Err: flip/{}", e));
  let pages = match state.pages.lock() {
    Ok(o) => o,
    Err(_) => {
      println!("Err: flip: pages lock");
      return respond_error(500, site, host, name, "", name);
    },
  };

  let page = match pages.get(name) {
    Some(o) => o,
    None => {
      println!("Err: flip: get page");
      return respond_error(500, site, host, name, "", name);
    },
  };

  HttpResponse::Ok().body(&page.html)
}


/*** state ********************************************************************/

struct Page
{
  name: String,
  link: String,
  //title: String,
  time: i64,
  lmod: i64,
  tags: Vec<String>,
  //scripts: Vec<String>,
  html: String,
}


struct State<'state>
{
  site: &'state str,
  dir: &'state str,
  scope: &'state str,
  pub_dir: &'state str,
  pub_scope: &'state str,
  pages: Mutex<BTreeMap<String,Page>>,
}

impl<'state> State<'state>
{
  pub fn up_state(&self, host: &str, link: &str, name: &str) ->
    Result<(),Error>
  {
    let mut pages = self.pages.lock()
      .or_else(|_| Err(Error("up_state: pages lock".to_string())))?;

    // empty name means refresh state of all pages
    if name.is_empty() {
      for (page_link, page_name, path) in &self.up_db(&mut pages)? {
        let page = self.up_page(host, &page_name, &page_link,
                                path.to_path_buf())?;
        pages.insert(page_name.to_string(), page);
      }
      return Ok(());
    }

    // refresh state of requested page only if necessary
    let page = pages.get(name)
      .ok_or_else(|| Error("up_state: get page".to_string()))?;
    if self.page_old(&page)? {
      let path = Path::new(&page_path(self.dir, &name)).to_path_buf();
      let page = self.up_page(host, name, link, path)?;
      pages.insert(name.to_string(), page);
    }
    return Ok(());
  }


  pub fn up_db(
    &self,
    pages: &mut BTreeMap<String,Page>
  ) -> Result<Vec<(String,String,PathBuf)>,Error>
  {
    // remove nonexistent pages
    // TODO: use BTreeMap.drain_filter() rather than reparse everything
    for (name, _) in pages.iter() {
      if !page_exists(self.dir, &name) {
        pages.clear();
        break;
      }
    }

    // detect files to parse anew
    let mut db = vec![];
    let files = fs::read_dir(self.dir)
      .or_else(|_| Err(Error("up_db: read_dir".to_string())))?;
    for f in files {
      // good file extension
      let file = f.or_else(|_| Err(Error("up_db: file".to_string())))?;
      let path = file.path();
      let mut name = path.to_str().unwrap().to_string();
      if !name.ends_with(".md") {
        continue;
      }

      // sane charset
      let caps = match Regex::new(&["^", self.dir, r"([^.]+)\.md$"].join(""))
        .unwrap().captures(&name)
      {
        Some(o) => o,
        None => { continue; },
      };
      name = match caps.get(1) {
        Some(o) => o.as_str().to_string(),
        None => { continue; },
      };
      let link = if name.eq("about") || name.eq("dev") {
        name.clone()
      } else {
        ["page/", &name].join("")
      };

      // page in state older than in db
      if pages.contains_key(name.as_str()) {
        let meta = file.metadata()
          .or_else(|_| Err(Error("up_db: file metadata".to_string())))?;
        let page = pages.get(&name)
          .ok_or_else(|| Error("up_db: get page".to_string()))?;
        if page_lmod(meta) <= page.lmod {
          continue;
        }
      }

      db.push((link, name, path));
    }

    Ok(db)
  }


  pub fn up_page(
    &self,
    host: &str,
    name: &str,
    link: &str,
    path: PathBuf,
  ) -> Result<Page,Error>
  {
    let (title, time, lmod, tags, scripts, md) = parse(name, path)?;
    let html_page = html_page(/* scope */ self.scope,
                              /* name  */ name,
                              /* title */ &title,
                              /* time  */ time,
                              /* lmod  */ lmod,
                              /* tags  */ &tags,
                              /* belly */ &md);

    Ok(Page {
      name: name.to_string(),
      link: link.to_string(),
      time: time,
      lmod: lmod,
      tags: tags,
      html: html(/* error   */ "",
                 /* site    */ self.site,
                 /* host    */ host,
                 /* link    */ &link,
                 /* query   */ "",
                 /* scripts */ &scripts,
                 /* html    */ &html_page),
    })
  }


  pub fn page_old(&self, page: &Page) ->
    Result<bool,Error>
  {
    let path = page_path(self.dir, &page.name);
    if !page_exists(self.dir, &page.name) {
      return Ok(false);
    }

    let meta = fs::metadata(path)
      .or_else(|_| Err(Error("page_old: file metadata".to_string())))?;
    Ok(page_lmod(meta) > page.lmod)
  }
}


/*** page helpers *************************************************************/

fn root_motd() ->
  String
{
  use rand::prelude::*;

  let mut rng = rand::thread_rng();
  let kaprekar = 6174;
  let answer = 42;
  let trinity = 3;
  let plutonium = kaprekar / answer / trinity;
  let n = (plutonium * plutonium - 1) / 2;

  std::iter::repeat(())
    .map(|()| rng.sample(rand::distributions::Alphanumeric))
    .map(char::from)
    .take(n)
    .collect()
}


fn pages_lists<'a>(
  input_query: &str,
  pages: &'a mut BTreeMap<String,Page>,
) -> (String,BTreeMap<&'a str,i16>,Vec<(&'a String,&'a Page)>)
{
  let query = match Regex::new(r"[?&]?tag=([0-9A-za-z]+)").unwrap()
    .captures(input_query)
  {
    Some(x) => match x.get(1) {
      Some(x) => clean(x.as_str()),
      None => String::new(),
    },
    None => String::new(),
  };

  let mut tags_take: BTreeSet<&str> = BTreeSet::new();
  let pages_list = pages.iter().filter(
    |(_, page)| {
      let mut take = true;
      if !query.is_empty() {
        take = page.tags.contains(&query);
      }
      if take {
        for tag in page.tags.iter() {
          tags_take.insert(&tag);
        }
      }
      take
    }).collect::<Vec<(&String,&Page)>>();

  let mut tags_list: BTreeMap<&str,i16> = BTreeMap::new();
  for (_, page) in pages.iter() {
    for tag in page.tags.iter() {
      if tags_take.contains(tag.as_str()) {
        *tags_list.entry(tag).or_insert(0) += 1;
      }
    }
  }

  (query, tags_list, pages_list)
}


fn flip_lick<'a>(
  req: &str,
  dir: &str,
  scope: &str,
) -> (bool,&'a str,String,String)
{
  let mut name = clean(&req[1..]);
  let max = 128;
  let mut bad = false;
  let mut msg = "";
  let msg_bad = "bad page request";
  let msg_miss = "page not found";

  // error: req too long
  let mut link = if max >= req.len() {
    name.clone()
  } else {
    bad = true;
    name.truncate(max - 1);
    msg = msg_bad;
    (&[&name, "\u{2026}"].join("")).to_string()
  };

  // trim "/page" prefix and set it to be the name
  if Some(0) == req.find(scope) {
    name = req.replacen(scope, "", 1);
  }

  // error: req not sane
  if !Regex::new(r"^[0-9A-Za-z_-]+$").unwrap().is_match(&name) {
    bad = true;
    msg = msg_bad;
    link = name.clone();
  }

  // error: req looks ok but does not point to a valid file
  if !bad && !page_exists(dir, &name) {
    bad = true;
    msg = msg_miss;
    link = name.clone();
  }

  (bad, msg, name, link)
}


/*** utility ******************************************************************/

fn parse(
  name: &str,
  path: PathBuf,
) -> Result<(String,i64,i64,Vec<String>,Vec<String>,String),Error>
{
  use std::io::BufRead;

  let mut title = Regex::new(r"[_-]+").unwrap()
    .replace(name, regex::NoExpand(" ")).to_string();
  let mut time = 0;
  let meta = fs::metadata(&path)
    .or_else(|_| Err(Error("parse: file metadata".to_string())))?;
  let sep = Regex::new(r",\s*").unwrap();
  let mut tags = vec![];
  let mut scripts = vec![];
  let mut md = String::new();
  let mut done_meta = false;
  let file = fs::File::open(path.as_path())
    .or_else(|_| Err(Error("parse: file open".to_string())))?;
  for l in io::BufReader::new(file).lines() {
    let line = l.unwrap();
    if line.eq(":::") {
      done_meta = true;
      continue;
    }
    if !done_meta {
      if line.is_empty() {
        continue;
      }
      match Regex::new(r"^\s*([^\s:]+):\s+(.*)$").unwrap().captures(&line) {
        Some(x) => {
          let cap = match x.get(2) {
            Some(x) => x.as_str(),
            None => { continue; },
          };
          match x.get(1) {
            Some(x) => match x.as_str() {
              "title" => title = cap.to_string(),
              "time" => time = cap.parse::<i64>().unwrap(),
              "tag" => tags = sep.split(cap).collect::<Vec<_>>().iter()
                .map(|s| (*s).to_string()).collect(),
              "script" => scripts = sep.split(cap).collect::<Vec<_>>().iter()
                .map(|s| (*s).to_string()).collect(),
              _ => { continue; },
            },
            None => { continue; },
          }
        },
        None => (),
      }
      continue;
    }
    md.push_str(&line);
    md.push_str("\n");
  }

  Ok((title, time, page_lmod(meta), tags, scripts, md_to_html(&md)))
}


fn md_to_html(md: &str) ->
  String
{
  use pulldown_cmark::{html, Options, Parser};

  let parser = Parser::new_ext(md, Options::all());
  let mut html_md = String::new();
  html::push_html(&mut html_md, parser);

  html_md
}


fn page_path(dir: &str, name: &str) ->
  String
{
  [dir, name, ".md"].join("")
}


fn page_exists(dir: &str, name: &str) ->
  bool
{
  Path::new(&page_path(dir, name)).is_file()
}

fn page_lmod(meta: fs::Metadata) ->
  i64
{
  meta.modified().unwrap()
    .duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap()
    .as_secs() as i64
}


fn timestamp(secs: i64) ->
  String
{
  let then = chrono::NaiveDateTime::from_timestamp(secs, 0);

  then.format("%Y-%m-%d").to_string()
}


fn respond_error(
  code: i32,
  site: &str,
  host: &str,
  link: &str,
  mut text: &str,
  more: &str,
) -> HttpResponse
{
  let (error, mut response) = match code {
    500 => {
      text = "trouble processing";
      ("500", HttpResponse::InternalServerError())
    },
    _ => {
      ("404", HttpResponse::NotFound())
    },
  };

  response.body(
    html(/* error   */ error,
         /* site    */ site,
         /* host    */ host,
         /* link    */ link,
         /* query   */ "",
         /* scripts */ &vec![],
         /* html    */ &html_pagelet(/* class */ error,
                                     /* title */ "oops",
                                     /* text  */ text,
                                     /* more  */ more)))
}


/*** html generation **********************************************************/

fn html(
  error: &str,           // error code
  site: &str,            // admin-defined site name
  dirty_host: &str,      // user-requested site name
  dirty_link: &str,      // user-requested path
  dirty_query: &str,     // user-requested query
  scripts: &Vec<String>, // optional page scripts
  belly: &str,           // the html going into body > .page > .page-body
) -> String
{
  let host = clean(dirty_host);
  let link = clean(dirty_link);
  let query = clean(dirty_query);

  [r#"<!doctype html>
<html lang="en">
<head>
"#,

   &html_analytics(site, &host),

   r#"  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link icon="/pub/favicon.ico">
  <link rel="stylesheet" href="/pub/css/style.css">
  <link rel="preconnect" href="https://fonts.gstatic.com">
  <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Roboto+Mono:ital,wght@0,400;0,700;1,400;1,700&display=swap">
"#,

   &html_scripts(scripts),

   r#"</head>
<body>
"#,

   &html_nav(&link),

   &belly,

   &html_foot(error, &query, &link),
   r#"</body>
"#].join("")
}


fn html_pages(
  query: &str,
  total: usize,
  tags: &BTreeMap<&str,i16>,
  pages: &Vec<(&String,&Page)>,
) -> String
{
  if pages.is_empty() {
    return html_pagelet("", "pages", "no pages to list", "");
  }

  let tags_list = tags.iter().map(
    |(tag, tag_count)|
    [r#"        <li><a class="tag"#,
     if tag.eq(&query) { " active" } else { "" },
     r#"" href="/pages?tag="#,
     tag,
     r#""><span class="tag-name">"#,
     tag,
     r#"</span><span class="tag-count">"#,
     &tag_count.to_string(),
     r#"</span></a></li>
"#].join("")).collect::<String>();

  let clear = &[r#"        <li><a class="tag clear"#,
                if query.is_empty() { " active" } else { "" },
                r#"" href="/pages"><span class="tag-name">all</span><span class="tag-count">"#,
                &total.to_string(),
                "</span></a></li>\n"].join("");

  let pages_list = pages.iter().map(
    |(name, page)| {
      let page_tags = if 0 == page.tags.len() {
        String::new()
      } else {
        [r#"          <ul class="page-tags">
"#,
         &page.tags.iter().map(
           |tag| [r#"            <li><a class="tag"#,
                  if query.eq(tag) { " active" } else { "" },
                  r#"" href="/pages?tag="#,
                  tag,
                  r#"">"#,
                  tag,
                  "</a></li>\n"].join("")).collect::<String>(),
         "          </ul>\n"].join("")
      };
      [r#"        <li>
          <div class="page-time">
            <div class="time">"#,
       &timestamp(page.time),
       r#"</div><div class="lmod">"#,
       &timestamp(page.lmod),
       r#"</div>
          </div>
          <a class="name" href="/"#,
       &page.link,
       r#"">"#,
       name,
       "</a>\n",
       &page_tags,
       "        </li>\n"].join("")
    }).collect::<String>();

  [r#"  <div class="page">
    <div class="page-head">
      <h1 class="title">pages</h1>
    </div>
    <div class="page-body">
      <ul class="tags">
"#,

   clear,

   &tags_list,

   r#"      </ul>
      <ul class="pages">
"#,

   &pages_list,

   r#"      </ul>
    </div>
  </div>
"#].join("")
}


fn html_page(
  scope: &str,
  name: &str,
  title: &str,
  time: i64,
  lmod: i64,
  tags: &Vec<String>,
  md: &str,
) -> String
{
  let page_tags = if 0 == tags.len() {
    String::new()
  } else {
    [r#"      <ul class="tags">
"#,
     &tags.iter().map(
       |tag|
       [r#"        <li><a class="tag" href=""#,
        scope,
        "s?tag=",
        tag,
        r#"">"#,
        tag,
        "</a></li>\n"].join("")).collect::<String>(),
     "      </ul>\n"].join("")
  };

  [r#"  <div class="page _"#,

   name,

   r#"">
    <div class="page-head">
      <h1 class="title">"#,

   title,

   "</h1>\n",

   &page_tags,

   r#"      <div class="time">"#,

   &timestamp(time),

   r#"</div>
      <div class="lmod">"#,

   &timestamp(lmod),

   r#"</div>
    </div>
    <div class="page-body">
"#,

   md,

   r#"    </div
  </div>
"#].join("")
}


fn html_pagelet(class: &str, title: &str, text: &str, more: &str) ->
  String
{
  let more_text = &[": <b>", more, "</b>"].join("");
  let class_text = &[" ", class].join("");

  [r#"  <div class="page"#,

   if class.is_empty() { "" } else { class_text },

   r#"">
    <div class="page-head">
      <h1 class="title">"#,

   title,

   r#"</h1>
    </div>
    <div class="page-body">
      <p>"#,

   text,

   if more.is_empty() { "" } else { more_text },

   r#"</p>
    </div>
  </div>
"#].join("")
}


fn html_analytics(site: &str, host: &str) ->
  String
{
  if host.eq(site) {
    r#"  <script async src="https://www.googletagmanager.com/gtag/js?id=G-PT10SS3WP3"></script><script>window.dataLayer = window.dataLayer || [];function gtag(){dataLayer.push(arguments);}gtag('js', new Date());gtag('config', 'G-PT10SS3WP3');</script>
"#.to_string()
  } else {
    String::new()
  }
}


fn html_scripts(scripts: &Vec<String>) ->
  String
{
  scripts.iter().map(
    |s| [r#"  <script src="/pub/js/"#,
         s,
         r#""></script>
"#].join("")).collect()
}


fn html_nav(link: &str) ->
  String
{
  [r#"  <div class="nav">
    <div class="root">
      <a "#,

   if link.eq("/") { r#"class="here" "# } else { "" },

   r#"href="/"><img class="cup" src="/pub/img/agaric-24.png">b</a>
    </div>
    <div class="links">
"#,

   &["about", "dev", "pages"].iter().map(
     |page|
     ["      <a ",
      if !link.is_empty() && page.eq(&link) {
        r#"class="here" "# } else { "" },
      r#"href="/"#,
      page,
      r#"">"#,
      page,
      "</a>\n"].join("")
   ).collect::<String>(),

   r#"    </div>
  </div>
"#].join("")
}


fn html_foot(error: &str, query: &str, link: &str) ->
  String
{
  if link.eq("/") {
    return String::new();
  }

  let dst = if query.is_empty() {
    ["/", link].join("")
  } else {
    ["/", link, "?tag=", query].join("")
  };
  let dsts = if query.is_empty() {
    ["/", link].join("")
  } else {
    ["/", link, r#"<span class="slip">?tag=</span>"#, query].join("")
  };
  let err = if error.is_empty() {
    String::new()
  } else {
    [r#"    <span class="error">"#,
     error,
     r#"</span>
"#].join("")
  };

  [r#"  <div class="foot">
"#,

   &err,

   r#"    <span class="stride"></span>
    <div class="lace"><a class="froot" href="/">root</a></span><a href=""#,

   &dst,

   r#"">"#,

   &dsts,

   r#"</a></div>
  </div>
"#].join("")
}

