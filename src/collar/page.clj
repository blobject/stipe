(ns collar.page
  (:require [collar.piece :as p]
            [collar.state :as s]
            [collar.util :as u]
            [clj-time.core :as tc]
            [clj-time.coerce :as tr]
            [clj-time.format :as tf]
            [hiccup.page :as h]
            [hiccup.util :as hu]
            [markdown.core :as md]))

(def pages-scope "/pages")
(def page-scope "/page")
(def specials ["about"])

;; template-related

(defn timestamp
  "Convert unix-epoch long to human-readable string."
  [time]
  (tf/unparse
   (tf/with-zone
     (tf/formatter "yyyy-MM-dd")
     (tc/time-zone-for-id "UTC"))
   (tr/from-long (.getTime (java.util.Date. time)))))

(defn get-tag-names
  "Parse tags from csv string."
  [tag-string]
  (-> tag-string (clojure.string/split #"\s*,\s*") sort))

(defn make-tag
  "Construct html out of tag."
  [tagname class]
  [:a {:href (if (= class "clear")
               pages-scope
               (str pages-scope "?tag=" (hu/url-encode tagname)))}
   [:div.tag (if class {:class class}) tagname]])

(defn template
  "Construct html out of page."
  [pre & text]
  (let [{:keys [short title time lmod tags]} pre]
    (h/html5
     (p/head short)
     (p/nav short)
     [:div.page
      [:div.page-head
       [:h1.title (or title short)]
       (if (seq tags)
         [:div.tags (->> tags
                         (map #(make-tag % "flat"))
                         (interpose ", "))])
       (if (some? time) [:div.time time])
       (if (and (some? lmod) (not= time lmod)) [:div.lmod lmod])]
      [:div.page-body text]])))

;; file-related

(defn special?
  "Whether page lives in root scope."
  [pagename]
  (some #{pagename} specials))

(defn valid-page? [file]
  "Whether file is a valid page."
  (and
   (.endsWith (clojure.string/lower-case (.getName file)) ".md")
   (> (.length file) 0)))

; get file representations of raw page data (static version)
(def page-files
  (->> (clojure.java.io/file u/db-path)
       .listFiles
       (filter valid-page?)
       sort))


; get file representations of raw page data (dynamic version)
(defn get-page-files []
  (->> (clojure.java.io/file u/db-path)
       .listFiles
       (filter valid-page?)
       sort))

(defn get-file
  "Read markdown data from file."
  [file]
  (md/md-to-html-string-with-meta (slurp file) :reference-links? true))

(defn get-page
  "Construct native representation of file."
  [file]
  (let [name (clojure.string/replace (.getName file) #"\.md$" "")
        short (clojure.string/replace name "-" " ")
        data (get-file file)
        meta (:metadata data)]
    {:name name
     :short short
     :title (if (seq (:title meta))
              (first (:title meta))
              short)
     :tags (if (seq (:keywords meta))
             (-> meta :keywords first get-tag-names)
             [])
     :time (if (seq (:date meta))
             (-> meta :date first Integer/parseInt (* 1000)))
     :lmod (.lastModified file)
     :text (:html data)}))

(defn get-pages
  "Get list of native representations of files."
  []
  (map get-page (if (s/state-changed?)
                  (get-page-files)
                  page-files)))

;; route-related

(defn flip
  "Construct html out of page."
  [pagename]
  (let [pages (get-pages)]
    (if (some #{pagename} (map :name pages))
      (let [page (first (filter #(= (:name %) pagename) pages))]
        (template
         {:short (:short page)
          :title (:title page)
          :time (if (:time page) (timestamp (:time page)))
          :lmod (timestamp (:lmod page))
          :tags (:tags page)}
         (:text page)))
      (str "\"" pagename "\" not found"))))

(def root-page
  (template
   {:short "root"
    :title "welcome"}
   [:p ":-:-)"]))

(defn pages-page
  "Construct html for 'pages' page."
  [query-tag]
  (let [all-pages (get-pages)
        pages (if query-tag
                (filter #(some #{query-tag} (:tags %)) all-pages)
                all-pages)
        page-count (count pages)]
    (template
     {:short "pages"}
     [:ul.tag-list
      (if query-tag (make-tag "tags" "clear") "tags") ": "
      (for [tag (->> all-pages (map :tags) flatten distinct sort)]
        (make-tag tag (if (= tag query-tag) "active")))]
     [:div.page-count
      page-count " page" (if (not= page-count 1) "s")]
     (if (> page-count 0)
       [:ul.page-list
        (for [page pages]
          (let [{:keys [name, short, tags]} page
                time (if (:lmod page)
                       (timestamp (:lmod page))
                       (timestamp (:time page)))
                link (if (special? name) name (str page-scope "/" name))]
            [:li
             [:a {:href link}
              [:span.page-name short] " " [:span.time time]]
             (if (seq tags)
               [:div.tags (for [tag tags] (make-tag tag nil))])]))]
       [:div.pageless [:em "no page tagged "] query-tag]))))
