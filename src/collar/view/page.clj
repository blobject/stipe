(ns collar.view.page
  (:require [collar.db :as db]
            [collar.view.piece :as piece]
            [collar.view.verse :as verse]
            [hiccup.page :as hiccup]))

(defn template-basic
  [title body]
  (hiccup/html5
   (verse/head title)
   verse/nav
   body))

(def root
  (template-basic
   "root"
   [:div.page
    [:h1 "the other head"]
    [:p ":-:-)"]]))

(def new
  (template-basic
   "new"
   [:div.page
    [:h1 "new"]
    [:textarea "hello"]]))

(def pages
  (template-basic
   "pages"
   [:div.page
    [:h1 "pages"]
    verse/search
    [:div.extract
     (map #(piece/clip %) (db/get-coll "entries"))]]))

(def tags
  (template-basic
   "tags"
   [:div.page
    [:h1 "tags"]
    (map #(piece/tag %) (db/get-coll "tags"))]))

(def who
  (template-basic
   "who"
   [:div.page
    [:h1 "who"]
    [:p "agaric"]]))
