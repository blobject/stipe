(ns collar.piece
  (:require [collar.util :as util]
            [hiccup.page :as hiccup]))

(defn tag [tag]
  [:div.tag
   [:div.name (:name tag)]])

(defn head [title]
  [:head
   [:title (str title " - alocy.be")]
   [:meta {:name "viewport"
           :content "width=device-width, initial-scale=1.0 maximum-scale=1.0, user-scalable=0"}]
   [:link {:href "https://fonts.googleapis.com/css?family=Source+Code+Pro|Source+Sans+Pro"
           :rel "stylesheet"}]
   (hiccup/include-css "/css/styles.css")])

(defn nav [title]
  [:div.nav
   [:div.root
    [:a {:href "/"
         :class (cond (= title "root") "here" :else nil)}
     (cond util/is-next? "dev.alocy.be" :else "alocybe")]]
   [:div.links
    [:a {:href "/about"
         :class (cond (= title "about") "here" :else nil)}
     "about"]
    [:a {:href "/pages"
         :class (cond (= title "pages") "here" :else nil)}
     "pages"]]])
