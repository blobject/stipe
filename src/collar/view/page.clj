(ns collar.view.page
  (:require [collar.db :as db]
            [collar.view.piece :as piece]
            [collar.view.verse :as verse]
            [config.core :refer [env]]
            [hiccup.page :as hiccup]))

(def entries (db/get-entries db/spec))
(def tags (db/get-tags db/spec))

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

(def draft
  (template-basic
   "draft"
   [:div.page
    [:h1 "draft"]
    [:div.draft-title "title" [:input]]
    [:div.draft-body "body" [:textarea "write"]]
    [:div.draft-tags [:select (map #(:name %) tags)]]]))

(def pages
  (template-basic
   "pages"
   [:div.page
    [:h1 "pages"]
    verse/search
    [:div.extract
     (map #(piece/clip %) entries)]]))

(def tags
  (template-basic
   "tags"
   [:div.page
    [:h1 "tags"]
    (map #(piece/tag %) tags)]))

(def who
  (template-basic
   "who"
   [:div.page
    [:h1 "who"]
    [:p (:body (db/get-entry-by-title db/spec {:title "who"}))]]))
