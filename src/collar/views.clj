(ns collar.views
  (:require [clojure.string :as str]
            [hiccup.page :as page]))

(defn head-for-page [title]
  [:head
   [:title (str title " - alocy.be")]
   (page/include-css "/css/styles.css")])

(def nav-for-page
  [:div#header-links
   [:a {:href "/"} "root"]
   " | "
   [:a {:href "/do"} "do"]
   " | "
   [:a {:href "/who"} "who"]
   " | "
   [:a {:href "/wip"} "wip"]])

(defn page-root []
  (page/html5
   (head-for-page "root")
   nav-for-page
   [:h1 "alocy.be"]
   [:p "the other head"]))

(defn page-do []
  (page/html5
   (head-for-page "do")
   nav-for-page
   [:h1 "do"]
   [:p "projects"]))

(defn page-who []
  (page/html5
   (head-for-page "who")
   nav-for-page
   [:h1 "who"]
   [:p "agaric"]))

(defn page-wip []
  (page/html5
   (head-for-page "wip")
   nav-for-page
   [:h1 "wip"]
   [:p "playground"]))
