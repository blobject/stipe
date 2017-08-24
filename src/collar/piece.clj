(ns collar.piece
  (:require [collar.util :as u]
            [hiccup.page :as h]))

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
   (h/include-css "/css/styles.css")])

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
