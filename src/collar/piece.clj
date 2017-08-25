(ns collar.piece
  (:require [collar.util :as u]
            [clojure.string :as s]
            [hiccup.page :as h]
            [hiccup.util :as hu]))

(def pages-scope "/pages")
(def page-scope "/page")
(def root-scoped #{"about" "apps"})

(defn head [title]
  [:head
   [:title (str title " - " u/site-name)]
   [:meta {:name "viewport"
           :content "width=device-width, initial-scale=1"}]
   [:link {:href "/favicon.ico" :rel "icon"}]
   (h/include-css "/css/style.css")
   [:link {:href "https://fonts.googleapis.com/css?family=Source+Code+Pro|Source+Sans+Pro"
           :rel "stylesheet"}]
   (if u/is-master?
     [:script "(function(i,s,o,g,r,a,m){i['GoogleAnalyticsObject']=r;i[r]=i[r]||function(){(i[r].q=i[r].q||[]).push(arguments)},i[r].l=1*new Date();a=s.createElement(o),m=s.getElementsByTagName(o)[0];a.async=1;a.src=g;m.parentNode.insertBefore(a,m)})(window,document,'script','https://www.google-analytics.com/analytics.js','ga');ga('create', 'UA-104505539-1', 'auto');ga('send', 'pageview');"])])

(defn nav [title]
  (let [here (fn [where] (if (= where title) "here"))]
    [:div.nav
     [:div.root
      (if u/is-next?
        [:span
         [:a {:href "/" :class (here "root")} "dev."]
         [:a {:href u/site-path} u/site-name]]
        [:a {:href "/" :class (here "root")}
         [:img {:class "cup" :src "/img/alocybe-24.png"}]
         "alocybe"])]
     [:div.links
      [:a {:href "/about" :class (here "about")} "about"]
      [:a {:href "/apps" :class (here "apps")} "apps"]
      [:a {:href "/pages" :class (here "pages")} "pages"]]]))

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
  (if name
    [:span [:em "no page called "] name]
    [:span [:em "page not found"]])
  )

(def root
  [:div {:class "welcome"}
   [:p
    (let [r #(rand-nth "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz")
          n (-> 6174 (/ 42) (/ 3) (Math/pow 2) int)]
      (->> r repeatedly (take n) (apply str)))]])

(defn taglist [query-tag count tags]
  [:ul.tag-list
   (if query-tag (create-tag "tags" "clear") "tags") ": "
   (map #(create-tag % (if (= % query-tag) "active")) (sort tags))
   [:div.page-count
    count " page" (if (not= count 1) "s")]])

(defn pagelist [query-tag count pages]
  [:ul.page-list
   {:class (if (= count 0) "pageless")}
   (if (= count 0)
     [:span [:em "no page tagged "] query-tag]
     (for [page pages]
       (let [{:keys [name, short, tags, lmod]} page
             time (if lmod
                    (u/timestamp lmod)
                    (u/timestamp (:time page)))
             link (if (root-scoped name)
                    name
                    (str page-scope "/" name))]
         [:li
          [:a {:href link}
           [:span.page-name short] " " [:span.time time]]
          (if (seq tags)
            [:div.tags (map #(create-tag % nil) (sort tags))])])))])
