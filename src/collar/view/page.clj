(ns collar.view.page
  (:require [collar.db :as db]
            [collar.view.piece :as piece]
            [collar.view.verse :as verse]
            [config.core :refer [env]]
            [hiccup.page :as hiccup]))

(def entries (db/get-entries db/spec))
(def tags (db/get-tags db/spec))

(defn template
  [short-title title & text]
  (hiccup/html5
   (verse/head short-title)
   (verse/nav short-title)
   [:div.page
    [:h1 (or title short-title)]
    text]))

(def root
  (template "root" "the other head"
   [:p ":-:-)"]))

(def draft
  (template "draft" false
   [:div.draft-title
    [:h2 "title"]
    [:input]]
   [:div.draft-body
    [:h2 "body"]
    [:textarea "write"]]
   [:div.draft-tags
    [:h2 "tags"]
    [:select (map #(:name %) tags)]]
   [:button.draft-submit "done"]))

(def pages
  (template "pages" false
   verse/search
   [:div.extract
    (map #(piece/clip %) entries)]))

(def who
  (template "who" false
   [:p (:body (db/get-entry-by-title db/spec {:title "who"}))]))
