(ns collar.piece
  (:require [collar.util :as u]
            [hiccup.page :as h]))

(defn head [title]
  [:head
   [:title (str title " - alocy.be")]
   [:meta {:name "viewport"
           :content "width=device-width, initial-scale=1.0 maximum-scale=1.0, user-scalable=0"}]
   [:link {:href "https://fonts.googleapis.com/css?family=Source+Code+Pro|Source+Sans+Pro"
           :rel "stylesheet"}]
   (h/include-css "/css/styles.css")])

(defn nav [title]
  [:div.nav
   [:div.root
    (if u/is-next?
      [:span
       [:a {:href "/"
            :class (if (= title "root") "here")}
        "dev."]
       [:a {:href u/site-path} "alocy.be"]]
      [:a {:href "/"
           :class (if (= title "root") "here")}
       "alocybe"])]
   [:div.links
    [:a {:href "/about"
         :class (if (= title "about") "here")}
     "about"]
    [:a {:href "/pages"
         :class (if (= title "pages") "here")}
     "pages"]]])
