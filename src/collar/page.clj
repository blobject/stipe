(ns collar.page
  (:require [clj-time.core :as tc]
            [clj-time.coerce :as tr]
            [clj-time.format :as tf]
            [collar.piece :as piece]
            [config.core :refer [env]]
            [hiccup.page :as hiccup]
            [markdown.core :as md]))

(def page-files-path "db/")

(def specials ["about"])

(defn timestamp [time]
  (tf/unparse
   (tf/with-zone
     (tf/formatter "yyyy-MM-dd")
     (tc/time-zone-for-id "UTC"))
   (tr/from-long (.getTime (java.util.Date. time)))))

(defn basename [name] (clojure.string/replace (.getName name) #"\.md$" ""))

(defn template
  [short-title title time lmod tags & text]
  (hiccup/html5
   (piece/head short-title)
   (piece/nav short-title)
   [:div.page
    [:div.page-head
     [:h1.title (or title short-title)]
     (if (some? tags) [:div.tags tags] nil)
     (if (some? time) [:div.time time] nil)
     (if (not= time lmod) [:div.lmod lmod] nil)]
    [:div.page-body text]]))

(def page-files
  (sort (filter #(.endsWith (clojure.string/lower-case %) ".md")
          (file-seq (clojure.java.io/file page-files-path)))))

(defn flip [page]
  (let [page-file (clojure.java.io/file (str page-files-path page ".md"))
        page-data (md/md-to-html-string-with-meta (slurp page-file) :reference-links? true)
        meta (:metadata page-data)
        short-title (clojure.string/replace page "_" " ")
        title (if (some? (:title meta)) (first (:title meta)) short-title)
        time (if (some? (:date meta))
               (timestamp (* (Long/parseLong (first (:date meta))) 1000)) nil)
        lmod (timestamp (.lastModified page-file))
        tags (if (some? (:keywords meta))
               (clojure.string/join ", " (:keywords meta)) nil)
        html (:html page-data)]
    (template
     short-title
     title
     time
     lmod
     tags
     html)))

(def root
  (template "root" "welcome" nil nil nil
   [:p ":-:-)"]))

(def pages
  (template "pages" nil nil nil nil
   [:ul.page-list
    (for [page-file page-files]
      (let [page-name (basename page-file)
            meta (:metadata (md/md-to-html-string-with-meta (slurp page-file)))]
        [:li
         [:a {:href (if (some #{page-name} specials)
                      page-name
                      (str "page/" page-name))}
          (str (clojure.string/replace page-name "_" " ") " ")
          [:span.time (timestamp (.lastModified page-file))]]
         (if (some? (:keywords meta))
           [:div.tags (for [tag (:keywords meta)] [:div.tag tag])]
           nil)]))]))
