(ns collar.view
  (:require [clojure.string :as str]
            [collar.util :as util]
            [config.core :refer [env]]
            [hiccup.page :as page]))

(defn head-for-page [title]
  [:head
   [:title (str title " - alocy.be")]
   [:meta {:name "viewport"
           :content "width=device-width, initial-scale=1.0 maximum-scale=1.0, user-scalable=0"}]
   [:link {:href "https://fonts.googleapis.com/css?family=Source+Code+Pro|Source+Sans+Pro"
           :rel "stylesheet"}]
   (page/include-css "/css/styles.css")])

(def nav-for-page
  [:div.nav
   [:div.root
    [:a {:href "/"} (cond util/is-next? "dev.alocy.be" :else "alocybe")]]
   [:div.links
    [:a {:href "/do"} "do"]
    [:a {:href "/who"} "who"]
    [:a {:href "/wip"} "wip"]]])

(defn page-root []
  (page/html5
   (head-for-page "root")
   nav-for-page
   [:div.page
    [:h1 "the other head"]
    [:p "stuff"]]))

(defn page-do []
  (page/html5
   (head-for-page "do")
   nav-for-page
   [:div.page
    [:h1 "do"]
    [:p "projects"]]))

(defn page-who []
  (page/html5
   (head-for-page "who")
   nav-for-page
   [:div.page
    [:h1 "who"]
    [:p "agaric"]]))

(defn page-wip []
  (page/html5
   (head-for-page "wip")
   nav-for-page
   [:div.page
    [:h1 "wip"]
    [:p "playground"]]))
