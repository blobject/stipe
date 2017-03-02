(ns collar.view.page
  (:require [collar.db :as db]
            [collar.view.piece :as piece]
            [collar.view.verse :as verse]
            [hiccup.page :as hiccup]))

(defn root []
  (hiccup/html5
   (verse/head "root")
   verse/nav
   [:div.page
    [:h1 "the other head"]
    [:p ":-:-)"]]))

(defn pages []
  (hiccup/html5
   (verse/head "pages")
   verse/nav
   [:div.page
    [:h1 "pages"]
    verse/search
    [:div.extract]]))

(defn tags []
  (hiccup/html5
   (verse/head "tags")
   verse/nav
   [:div.page
    [:h1 "tags"]
    (map #(piece/tag %) (db/get-coll "tags"))]))

(defn who []
  (hiccup/html5
   (verse/head "who")
   verse/nav
   [:div.page
    [:h1 "who"]
    [:p "agaric"]]))
