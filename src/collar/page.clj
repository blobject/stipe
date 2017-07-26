(ns collar.page
  (:require [collar.piece :as piece]
            [collar.state :refer [STATE]]
            [clj-time.core :as tc]
            [clj-time.coerce :as tr]
            [clj-time.format :as tf]
            [hiccup.page :as hiccup]
            [markdown.core :as md]))

;; file-related

; pages that live on root scope
(def specials ["about"])
(defn special? [pagename] (some #{pagename} specials))

; directory that holds raw page data
(def page-files-path "db/")

; detect state change
; - currently, only the "last-modified time" of db folder is saved
;   basically a flag for whether to re-read files in db
(defn state-changed? []
  (let [db-mod (.lastModified (clojure.java.io/file page-files-path))]
    (if (not= (:db-mod @STATE) db-mod)
      (do
        (swap! STATE assoc :db-mod db-mod)
        true)
      false)))

; "a page" filter
(defn page-filter [file]
  (and
   (.endsWith (clojure.string/lower-case (.getName file)) ".md")
   (> (.length file) 0)))

; get file representations of raw page data, static version
(def page-files
  (sort (filter page-filter
                (.listFiles (clojure.java.io/file page-files-path)))))

; get file representations of raw page data, dynamic version
(defn get-page-files []
  (sort (filter page-filter
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
(defn get-pages []
  (map #(get-page %) (if (state-changed?)
                       (get-page-files)
                       page-files)))

;; template-related

; construct html view out of page
(defn template [short title time lmod tags & text]
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
(defn flip [pagename]
  (let [pages (get-pages)]
    (if (some #{pagename} (map #(:name %) pages))
      (let [page (first (filter #(= (:name %) pagename) pages))
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
      (str "\"" pagename "\" not found"))))

; spit out html view of "root" page
(def root-page
  (template
   "root" "welcome" nil nil nil
   [:p ":-:-)"]))

; spit out html view of "pages" page
(defn pages-page []
  (template
   "pages" nil nil nil nil
   [:ul.page-list
    (for [page (get-pages)]
      (let [name (:name page)
            short (:short page)
            lmod (:lmod page)
            tags (:tags page)
            link (if (special? name) name (str "page/" name))]
        [:li
         [:a {:href link} short " " [:span.time (timestamp lmod)]]
         (if (seq tags)
           [:div.tags (for [tag tags] [:div.tag tag])]
           nil)]
        ))]))
