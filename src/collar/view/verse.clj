(ns collar.view.verse
  (:require [clojure.string :as str]
            [collar.util :as util]
            [hiccup.page :as hiccup]))

(defn head [title]
  [:head
   [:title (str title " - alocy.be")]
   [:meta {:name "viewport"
           :content "width=device-width, initial-scale=1.0 maximum-scale=1.0, user-scalable=0"}]
   [:link {:href "https://fonts.googleapis.com/css?family=Source+Code+Pro|Source+Sans+Pro"
           :rel "stylesheet"}]
   (hiccup/include-css "/css/styles.css")])

(def nav
  [:div.nav
   [:div.root
    [:a {:href "/"} (cond util/is-next? "dev.alocy.be" :else "alocybe")]]
   [:div.links
    [:a {:href "/pages"} "pages"]
    [:a {:href "/tags"} "tags"]
    [:a {:href "/who"} "who"]]])

(def search
  [:div.search
   [:div.form
    [:input {:placeholder "filter"}]
    [:button {:type "submit"} "apply"]]])
