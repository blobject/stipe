(ns collar.page
  (:require [collar.piece :as p]
            [collar.state :as state]
            [collar.util :as u]
            [clojure.string :as s]))

(defn get-tag-names [tag-string]
  (-> tag-string (s/split #"\s*,\s*") sort))

(defn get-page [raw-page]
  (let [{:keys [data name lmod]} raw-page
        {:keys [title date keywords]} (:metadata data)
        short (s/replace name "-" " ")]
    {:name name
     :short short
     :title (if (seq title) (first title) short)
     :tags (if (seq keywords) (-> keywords first get-tag-names) [])
     :time (if (seq date) (-> date first Integer/parseInt (* 1000)))
     :lmod lmod
     :text (:html data)}))

(defn get-pages []
  (:data (state/upstate!)))

(defn flip [name]
  (let [which (-> (get-pages) (get name))
        page (if which (get-page which))]
    (if-not page
      (p/create-page
       {:short "not found"}
       (p/notfound name))
      (p/create-page
       {:short (:short page)
        :title (:title page)
        :time (if (:time page) (u/timestamp (:time page)))
        :lmod (u/timestamp (:lmod page))
        :tags (:tags page)}
       (:text page)))))

(def flip-root
  (p/create-page
   {:short "root"
    :title "welcome"}
   p/root))

(defn flip-pages [query-tag]
  (let [pages (->> (get-pages) (map peek) (map get-page))
        tags (->> pages (map :tags) flatten distinct sort)
        which (if query-tag
                (filter #(some #{query-tag} (:tags %)) pages)
                pages)
        count (count which)]
    (p/create-page
     {:short "pages"}
     (p/taglist query-tag count tags)
     (p/pagelist query-tag count which))))
