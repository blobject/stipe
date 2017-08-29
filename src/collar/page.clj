(ns collar.page
  (:require [collar.piece :as p]
            [collar.state :as state]
            [collar.util :as u]
            [clojure.string :as s]))

(defn get-page [raw-page]
  (let [{:keys [data name lmod]} raw-page
        {:keys [title date keywords]} (:metadata data)
        short (s/replace name "-" " ")]
    {:name name
     :short short
     :title (if (seq title) (first title) short)
     :tags (if (seq keywords)
             (-> keywords first (s/split #"\s*,\s*") set)
             #{})
     :time (if (seq date) (-> date first Integer/parseInt (* 1000)))
     :lmod lmod
     :text (:html data)}))

(defn get-pages []
  (state/upstate))

(defn flip [name]
  (let [which (-> (get-pages) (get name))]
    (if-not which
      (p/create-page
       {:short "not found"}
       nil
       (p/not-found name))
      (let [{:keys [short title time lmod tags text]} (get-page which)]
        (p/create-page
         {:short short
          :title title
          :time (if time (u/timestamp time))
          :lmod (u/timestamp lmod)
          :tags tags}
         (p/foot name)
         text)))))

(def flip-root
  (p/create-page
   {:short "root"
    :title "welcome"}
   nil
   p/root))

(defn flip-pages [query-tag]
  (let [pages (->> (get-pages) (map peek) (map get-page))
        tags (->> pages (map :tags) (apply clojure.set/union))
        which (if query-tag
                (filter #(get (:tags %) query-tag) pages)
                pages)
        count (count which)
        short "pages"]
    (p/create-page
     {:short short}
     (p/foot short)
     (p/tags-list query-tag count tags)
     (p/pages-list query-tag count which))))
