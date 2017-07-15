(ns collar.page
  (:require [clj-time.core :as tc]
            [clj-time.coerce :as tr]
            [clj-time.format :as tf]
            [collar.piece :as piece]
            [config.core :refer [env]]
            [hiccup.page :as hiccup]
            [markdown.core :as md]))

(defn timestamp [time]
  (tf/unparse
   (tf/with-zone
     (tf/formatter "yyyy-MM-dd")
     (tc/time-zone-for-id "UTC"))
   (tr/from-long (.getTime time))))

(defn template
  [short-title title time & text]
  (hiccup/html5
   (piece/head short-title)
   (piece/nav short-title)
   [:div.page
    [:div.page-head
     [:h1.title (or title short-title)]
     [:div.time time]]
    [:div.page-body text]]))

(def page-paths
  (sort (filter #(.endsWith (clojure.string/lower-case %) ".md")
          (file-seq (clojure.java.io/file "resources/data/")))))

(defn flip [page]
  (let [page-data (md/md-to-html-string-with-meta (slurp (str "resources/data/" page ".md")))
        meta (:metadata page-data)
        time (:date meta)
        html (:html page-data)
        short-title (clojure.string/replace page "_" " ")]
    (template
     short-title
     (if (some? (:title meta))
       (first (:title meta))
       short-title)
     (if (some? time)
       (timestamp (java.util.Date. (* (Long/parseLong (first time)) 1000)))
       nil)
     html)))

(def root
  (template "root" "welcome" nil
   [:p ":-:-)"]))

(def pages
  (template "pages" nil nil
   [:div.extract
    [:ul
     (for [page-name (map #(clojure.string/replace % #"^.*/(.*)\.md$" "$1") page-paths)]
       [:li [:a {:href page-name}
             (clojure.string/replace page-name "_" " ")]])]]))
