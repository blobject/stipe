(ns collar.piece
  (:require [collar.util :as u]
            [clojure.string :as s]
            [hiccup.page :as h]
            [hiccup.util :as hu]))

(def pages-scope "/pages")
(def page-scope "/page")
(def root-scoped ["about" "apps"])

(defn root-scoped? [pagename]
  (some #{pagename} root-scoped))

(defn here [path contra]
  (if (= path contra) "here"))

(defn head [title]
  [:head
   [:title (str title " - " u/site-name)]
   [:meta {:name "viewport"
           :content "width=device-width, initial-scale=1.0 maximum-scale=1.0, user-scalable=0"}]
   (if u/is-master?
     [:script "(function(i,s,o,g,r,a,m){i['GoogleAnalyticsObject']=r;i[r]=i[r]||function(){(i[r].q=i[r].q||[]).push(arguments)},i[r].l=1*new Date();a=s.createElement(o),m=s.getElementsByTagName(o)[0];a.async=1;a.src=g;m.parentNode.insertBefore(a,m)})(window,document,'script','https://www.google-analytics.com/analytics.js','ga');ga('create', 'UA-104505539-1', 'auto');ga('send', 'pageview');"])
   [:link {:href "https://fonts.googleapis.com/css?family=Source+Code+Pro|Source+Sans+Pro"
           :rel "stylesheet"}]
   (h/include-css "/css/style.css")])

(defn nav [title]
  [:div.nav
   [:div.root
    (if u/is-next?
      [:span
       [:a {:href "/" :class (here title "root")} "dev."]
       [:a {:href u/site-path} u/site-name]]
      [:a {:href "/" :class (here title "root")} "alocybe"])]
   [:div.links
    [:a {:href "/about" :class (here title "about")} "about"]
    [:a {:href "/apps" :class (here title "apps")} "apps"]
    [:a {:href "/pages" :class (here title "pages")} "pages"]]])

(defn create-tag [tagname class]
  [:a {:href (if (= class "clear")
               pages-scope
               (str pages-scope "?tag=" (hu/url-encode tagname)))}
   [:div.tag (if class {:class class}) tagname]])

(defn create-page [pre & text]
  (let [{:keys [short title time lmod tags]} pre]
    (h/html5
     (head short)
     (nav short)
     [:div {:class (str "page _" (s/replace short " " "-"))}
      [:div.page-head
       [:h1.title (or title short)]
       (if (seq tags)
         [:div.tags (->> tags
                         (map #(create-tag % "flat"))
                         (interpose ", "))])
       (if (some? time) [:div.time time])
       (if (and (some? lmod) (not= time lmod)) [:div.lmod lmod])]
      [:div.page-body text]])))

(defn notfound [name]
  [:span [:em "no page called "] name])

(def root
  [:div {:class "welcome"}
   [:p
    (let [r #(rand-nth "0123456789BCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz")]
      (apply str (take 6174 (repeatedly r))))]])

(defn taglist [query-tag page-count tags]
  [:ul.tag-list
   (if query-tag (create-tag "tags" "clear") "tags") ": "
   (for [tag tags]
     (create-tag tag (if (= tag query-tag) "active")))
   [:div.page-count
    page-count " page" (if (not= page-count 1) "s")]])

(defn pagelist [query-tag page-count pages]
  [:ul.page-list
   {:class (if (= page-count 0) "pageless")}
   (if (= page-count 0)
     [:span [:em "no page tagged "] query-tag]
     (for [page pages]
       (let [{:keys [name, short, tags, lmod]} page
             time (if lmod
                    (u/timestamp lmod)
                    (u/timestamp (:time page)))
             link (if (root-scoped? name)
                    name
                    (str page-scope "/" name))]
         [:li
          [:a {:href link}
           [:span.page-name short] " " [:span.time time]]
          (if (seq tags)
            [:div.tags (for [tag tags] (create-tag tag nil))])])))])
