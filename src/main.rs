use actix_web::{web, HttpRequest, HttpResponse, Responder, Result};
use actix_files::NamedFile;
use ammonia::clean;
use regex::Regex;
use std::collections::{BTreeSet, BTreeMap};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;


/*** structures ***************************************************************/

struct State {
  site: &'static str,
  dir: &'static str,
  scope: &'static str,
  pub_dir: &'static str,
  pub_scope: &'static str,
  pages: Mutex<BTreeMap<String,Page>>,
}


struct Page {
  link: String,
  //title: String,
  time: i64,
  lmod: i64,
  tags: Vec<String>,
  //scripts: Vec<String>,
  html: String,
}


/*** serving ******************************************************************/

#[actix_web::main]
async fn main() -> io::Result<()> {
  use actix_web::{guard, App, HttpServer};

  let site = "b.agaric.net";
  let dir = "db/";
  let scope = "/page";
  let pub_dir = "pub";
  let pub_scope = "/pub";
  let mut init = BTreeMap::new();
  up_state(/* site  */ site,
           /* host  */ "b.agaric.net",
           /* dir   */ dir,
           /* scope */ scope,
           /* link  */ "",
           /* name  */ "", // up_db() as well
           /* pages */ &mut init);
  let state = web::Data::new(State {
    site: site,
    dir: dir,
    scope: scope,
    pub_dir: pub_dir,
    pub_scope: pub_scope,
    pages: Mutex::new(init),
  });

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


/*** routing for fixed pages **************************************************/

async fn route_fav(_: HttpRequest) -> Result<NamedFile> {

  Ok(NamedFile::open("pub/favicon.ico")?)
}


async fn route_root(
  state: web::Data<State>,
  req: HttpRequest,
) -> impl Responder {

  HttpResponse::Ok().body(
    html(/* error   */ false,
         /* site    */ state.site,
         /* host    */ req.connection_info().host(),
         /* link    */ "/",
         /* query   */ "",
         /* scripts */ &vec![],
         /* html    */ &pagelet_html(/* class */ "welcome",
                                     /* title */ "welcome",
                                     /* text  */ &root_motd(),
                                     /* more  */ "")))
}


async fn route_pages(
  state: web::Data<State>,
  req: HttpRequest,
) -> impl Responder {

  let mut pages = state.pages.lock().unwrap();
  let site = state.site;
  let conn = req.connection_info();
  let host = conn.host();
  up_state(/* site  */ site,
           /* host  */ host,
           /* dir   */ state.dir,
           /* scope */ state.scope,
           /* link  */ "",
           /* name  */ "", // up_db() as well
           /* pages */ &mut pages);

  let pages_total = pages.len();
  let (query, tags_list, pages_list) =
    &pages_lists(req.query_string(), &mut pages);
  if 0 == pages_list.len() && 0 < pages_total {
    return HttpResponse::NotFound().body(
      html(/* error   */ true,
           /* site    */ site,
           /* host    */ host,
           /* link    */ "pages",
           /* query   */ query,
           /* scripts */ &vec![],
           /* html    */ &pagelet_html(/* class */ "error",
                                       /* title */ "oops",
                                       /* text  */ "no pages tagged",
                                       /* more  */ query)));
  }

  HttpResponse::Ok().body(
    html(/* error   */ false,
         /* site    */ site,
         /* host    */ host,
         /* link    */ "pages",
         /* query   */ query,
         /* scripts */ &vec![],
         /* html    */ &pages_html(/* query */ query,
                                   /* total */ pages_total,
                                   /* tags  */ tags_list,
                                   /* pages */ pages_list)))
}


/*** routing for dynamic (markdown-parsed) pages ******************************/

async fn flip(
  state: web::Data<State>,
  req: HttpRequest,
) -> impl Responder {

  let mut pages = state.pages.lock().unwrap();
  let conn = req.connection_info();
  let host = conn.host();

  let dir = state.dir;
  let scope = state.scope;
  let (bad, msg, name, text) = &flip_lick(req.match_info().path(), dir,
                                          &[scope, "/"].join(""));
  if *bad {
    return HttpResponse::NotFound().body(
      html(/* error   */ true,
           /* site    */ state.site,
           /* host    */ host,
           /* link    */ text,
           /* query   */ "",
           /* scripts */ &vec![],
           /* html    */ &pagelet_html(/* class */ "error",
                                       /* title */ "oops",
                                       /* text  */ *msg,
                                       /* more  */ text)));
  }

  // ensure state of requested page is up to date
  //       site        host  dir  scope        link  name          pages
  up_state(state.site, host, dir, state.scope, &text, name, &mut pages);

  HttpResponse::Ok().body(&pages.get(name).unwrap().html)
}


/*** state ********************************************************************/

fn up_state(
  site: &str,
  host: &str,
  dir: &str,
  scope: &str,
  link: &str,
  name: &str,
  pages: &mut BTreeMap<String,Page>,
) {
  // empty name means refresh state of all pages
  if name.is_empty() {
    for (page_link, page_name, path) in &up_db(dir, pages) {
      pages.insert(page_name.to_string(),
                   up_page(site, host, scope, &page_link, &page_name,
                           path.to_path_buf()));
    }

  // refresh state of requested page only if necessary
  } else if page_old(dir, &name, pages) {
    let path = Path::new(&page_path(dir, &name)).to_path_buf();
    pages.insert(name.to_string(),
                 up_page(site, host, scope, link, name, path));
  }
}


fn up_db(
  dir: &str,
  pages: &mut BTreeMap<String,Page>,
) -> Vec<(String,String,PathBuf)> {

  // remove nonexistent pages
  // TODO: use BTreeMap.drain_filter() rather than reparse everything
  for (name, _) in pages.iter() {
    if !page_exists(&page_path(dir, &name)) {
      pages.clear();
      break;
    }
  }

  // detect files to parse anew
  if !Path::new(dir).is_dir() {
    return vec![];
  }

  let mut db = vec![];
  for f in fs::read_dir(dir).unwrap() {
    let file = f.unwrap();
    let path = file.path();
    let mut name = path.to_str().unwrap().to_string();
    if !name.ends_with(".md") {
      continue;
    }
    name = Regex::new(&["^", dir, r"([^.]+)\.md$"].join(""))
      .unwrap().captures(&name).unwrap().get(1).unwrap().as_str().to_string();
    let link = if name.eq("about") || name.eq("dev") {
      name.clone()
    } else {
      ["page/", &name].join("")
    };
    if pages.contains_key(name.as_str()) &&
      page_lmod(file.metadata().unwrap()) <= pages.get(&name).unwrap().lmod
    {
      continue;
    }
    db.push((link, name, path));
  }

  db
}


fn up_page(
  site: &str,
  host: &str,
  scope: &str,
  link: &str,
  name: &str,
  path: PathBuf,
) -> Page {

  let (title, time, lmod, tags, scripts, md) = parse(name, path);
  let page_html = page_html(/* scope */ scope,
                            /* name  */ name,
                            /* title */ &title,
                            /* time  */ time,
                            /* lmod  */ lmod,
                            /* tags  */ &tags,
                            /* belly */ &md);

  Page {
    link: link.to_string(),
    time: time,
    lmod: lmod,
    tags: tags,
    html: html(/* error   */ false,
               /* site    */ site,
               /* host    */ host,
               /* link    */ &link,
               /* query   */ "",
               /* scripts */ &scripts,
               /* html    */ &page_html),
  }
}


/*** fixed-page helpers *******************************************************/

fn root_motd() -> String {
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
) -> (String, BTreeMap<&'a str,i16>, Vec<(&'a String,&'a Page)>) {

  let query = match Regex::new(r"[?&]?tag=([0-9A-za-z]+)").unwrap()
    .captures(input_query)
  {
    Some(x) => clean(x.get(1).unwrap().as_str()).to_string(),
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
  path: &str,
  dir: &str,
  scope: &str,
) -> (bool, &'a str, String, String) {

  let mut name = clean(&path[1..]);
  let mut text = String::new();
  let max = 128;
  let mut bad = false;
  let mut msg = "";
  let msg_bad = "bad page request";
  let msg_miss = "page not found";

  // error: path too long
  if max < path.len() {
    bad = true;
    name.truncate(max - 1);
    msg = msg_bad;
    text = (&[&name, "\u{2026}"].join("")).to_string();
  }
  // non-error: trim "/page" prefix
  if Some(0) == path.find(scope) {
    name = path.replacen(scope, "", 1);
  }
  // error: path not sane
  if !Regex::new(r"^[0-9A-Za-z_-]+$").unwrap().is_match(&name) {
    bad = true;
    msg = msg_bad;
    text = name.clone();
  }
  // error: path looks ok but does not point to a valid file
  if !bad && !page_exists(&page_path(dir, &name)) {
    bad = true;
    msg = msg_miss;
    text = name.clone();
  }

  (bad, msg, name, text)
}


/*** utility ******************************************************************/

fn parse(
  name: &str,
  path: PathBuf,
) -> (String, i64, i64, Vec<String>, Vec<String>, String) {
  use std::io::BufRead;

  let mut title = Regex::new(r"[_-]+").unwrap()
    .replace(name, regex::NoExpand(" ")).to_string();
  let mut time = 0;
  let lmod = page_lmod(fs::metadata(&path).unwrap());
  let sep = Regex::new(r",\s*").unwrap();
  let mut tags = vec![];
  let mut scripts = vec![];
  let mut md = String::new();
  let mut done_meta = false;
  for l in io::BufReader::new(fs::File::open(path.as_path()).unwrap()).lines() {
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
          let cap = x.get(2).unwrap().as_str();
          match x.get(1).unwrap().as_str() {
            "title" => title = cap.to_string(),
            "time" => time = cap.parse::<i64>().unwrap(),
            "tag" => tags = sep.split(cap).collect::<Vec<_>>().iter()
              .map(|s| (*s).to_string()).collect(),
            "script" => scripts = sep.split(cap).collect::<Vec<_>>().iter()
              .map(|s| (*s).to_string()).collect(),
            _ => ()
          }
        }
        None => ()
      }
      continue;
    } else {
      md.push_str(&line);
      md.push_str("\n");
    }
  }

  (title, time, lmod, tags, scripts, md_to_html(&md))
}


fn md_to_html(md: &str) -> String {
  use pulldown_cmark::{html, Options, Parser};

  let parser = Parser::new_ext(md, Options::all());
  let mut md_html = String::new();
  html::push_html(&mut md_html, parser);

  md_html
}


fn page_old(
  dir: &str,
  name: &str,
  pages: &BTreeMap<String,Page>,
) -> bool {

  let path = page_path(dir, name);
  if !page_exists(&path) {
    return false;
  }

  let page = pages.get(name).unwrap();

  page_lmod(fs::metadata(path).unwrap()) > page.lmod
}


fn page_path(dir: &str, name: &str) -> String {

  [dir, name, ".md"].join("")
}


fn page_lmod(meta: fs::Metadata) -> i64 {

  meta.modified().unwrap()
    .duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap()
    .as_secs() as i64
}


fn page_exists(path: &str) -> bool {

  Path::new(path).exists()
}


fn timestamp(secs: i64) -> String {

  let then = chrono::NaiveDateTime::from_timestamp(secs, 0);

  then.format("%Y-%m-%d").to_string()
}


/*** html generation **********************************************************/

fn html(
  error: bool,           // 404 error
  site: &str,            // admin-defined site name
  dirty_host: &str,      // detected site name
  dirty_link: &str,      // user-requested path
  dirty_query: &str,     // user-requested query
  scripts: &Vec<String>, // optional page scripts
  belly: &str,           // the html going into body > .page > .page-body
) -> String {

  let link = clean(dirty_link);

  [r#"<!doctype html>
<html lang="en">
<head>
"#,

   &analytics_html(site, &clean(dirty_host)),

   r#"  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link icon="/pub/favicon.ico">
  <link rel="stylesheet" href="/pub/css/style.css">
  <link rel="preconnect" href="https://fonts.gstatic.com">
  <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Roboto+Mono:ital,wght@0,400;0,700;1,400;1,700&display=swap">
"#,

   &scripts_html(scripts),

   r#"</head>
<body>
"#,

   &nav_html(&link),

   &belly,

   &foot_html(error, &clean(dirty_query), &link),
   r#"</body>
"#].join("")
}


fn pages_html(
  query: &str,
  total: usize,
  tags: &BTreeMap<&str,i16>,
  pages: &Vec<(&String,&Page)>,
) -> String {

  if pages.is_empty() {
    return pagelet_html("", "pages", "no pages to list", "");
  }

  let head_tags = tags.iter().map(
    |(tag, tag_count)|
    [r#"<li><a class="tag"#,
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
                r#"" href="/pages"><span class="tag-name">all</span><span class="tag-count">
"#,
                &total.to_string(),
                "</span></a></li>\n        "].join("");
  let tags_html = [r#"      <ul class="tags">
"#,
                   clear,
                   &head_tags,
                   "      </ul>\n"].join("");
  let pages_html = pages.iter().map(
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
"#,
   &tags_html,
   r#"      <ul class="pages">
"#,
   &pages_html,
   r#"      </ul>
    </div>
  </div>
"#].join("")
}


fn page_html(
  scope: &str,
  name: &str,
  title: &str,
  time: i64,
  lmod: i64,
  tags: &Vec<String>,
  md: &str,
) -> String {

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


fn pagelet_html(class: &str, title: &str, text: &str, more: &str) -> String {

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


fn analytics_html(site: &str, host: &str) -> String {

  if host.eq(site) {
    r#"  <script async src="https://www.googletagmanager.com/gtag/js?id=G-PT10SS3WP3"></script><script>window.dataLayer = window.dataLayer || [];function gtag(){dataLayer.push(arguments);}gtag('js', new Date());gtag('config', 'G-PT10SS3WP3');</script>
"#.to_string()
  } else {
    String::new()
  }
}


fn scripts_html(scripts: &Vec<String>) -> String {

  scripts.iter().map(
    |s| [r#"  <script src="/pub/js/"#,
         s,
         r#""></script>
"#].join("")).collect()
}


fn nav_html(link: &str) -> String {

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


fn foot_html(error: bool, query: &str, link: &str) -> String {

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

  [r#"  <div class="foot">
"#,
   if error { r#"    <span class="error">404</span>
"# } else { "" },
   r#"    <span class="stride"></span>
    <div class="lace"><a class="froot" href="/">root</a></span><a href=""#,
   &dst,
   r#"">"#,
   &dsts,
   r#"</a></div>
  </div>
"#].join("")
}

