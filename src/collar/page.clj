(ns collar.page
  (:require [clj-time.core :as tc]
            [clj-time.coerce :as tr]
            [clj-time.format :as tf]
            [collar.piece :as piece]
            [config.core :refer [env]]
            [hiccup.page :as hiccup]
            [markdown.core :as md]))

;; file-related

; names of pages that live on root scope
(def specials ["about"])

; directory that holds raw page data
(def page-files-path "db/")

; get file representations of raw page data
(def page-files
  (sort (filter #(.endsWith (clojure.string/lower-case (.getName %)) ".md")
                (.listFiles (clojure.java.io/file page-files-path)))))

; get markdown data from a page file
(defn get-file [file]
  (md/md-to-html-string-with-meta (slurp file) :reference-links? true))

; get native representation of a page file
(defn get-page [file]
  (let [name (clojure.string/replace (.getName file) #"\.md$" "")
        short (clojure.string/replace name "_" " ")
        data (get-file file)
        meta (:metadata data)]
    {:name name
     :short short
     :title (if (seq (:title meta))
              (first (:title meta))
              short)
     :tags (if (seq (:keywords meta))
             (clojure.string/split (first (:keywords meta)) #"\s*,\s*")
             [])
     :time (if (seq (:date meta))
             (* (Integer/parseInt (first (:date meta))) 1000)
             nil)
     :lmod (.lastModified file)
     :text (:html data)}))

; get native representations of page files
(defn get-pages [files]
  (map #(get-page %) files))

;; template-related

; construct html view out of page
(defn template
  [short title time lmod tags & text]
  (hiccup/html5
   (piece/head short)
   (piece/nav short)
   [:div.page
    [:div.page-head
     [:h1.title (or title short)]
     (if (some? tags) [:div.tags tags] nil)
     (if (some? time) [:div.time time] nil)
     (if (and (some? lmod) (not= time lmod)) [:div.lmod lmod] nil)]
    [:div.page-body text]]))

; convert unix-epoch-long to human-readable string
(defn timestamp [time]
  (tf/unparse
   (tf/with-zone
     (tf/formatter "yyyy-MM-dd")
     (tc/time-zone-for-id "UTC"))
   (tr/from-long (.getTime (java.util.Date. time)))))

;; route-related

; spit out html view of page
(defn flip [page-name]
  (let [pages (get-pages page-files)]
    (if (some #{page-name} (map #(:name %) pages))
      (let [page (first (filter #(= (:name %) page-name) pages))
            short (:short page)
            title (:title page)
            time (if (:time page)
                   (timestamp (:time page))
                   nil)
            lmod (timestamp (:lmod page))
            tags (if (seq (:tags page))
                   (clojure.string/join ", " (:tags page))
                   nil)
            text (:text page)]
        (template short title time lmod tags text))
      (str "\"" page-name "\" not found"))))

; spit out html view of "root" page
(def root
  (template "root" "welcome" nil nil nil
   [:p ":-:-)"]))

; spit out html view of "pages" page
(def pages
  (template "pages" nil nil nil nil
   [:ul.page-list
    (for [page (get-pages page-files)]
      (let [name (:name page)
            short (:short page)
            title (:title page)
            lmod (:lmod page)
            tags (:tags page)
            link (if (some #{name} specials)
                   name
                   (str "page/" name))]
        [:li
         [:a {:href link} short " " [:span.time (timestamp lmod)]]
         (if-not (seq tags) nil
           [:div.tags (for [tag tags] [:div.tag tag])])]
        ))]))
