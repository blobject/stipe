(ns collar.db
  (:require [clj-time.core :as tc]
            [clj-time.coerce :as tr]
            [clj-time.format :as tf]
            [datomic.api :as d]
            [markdown.core :as md]))

(def page-files-path "db/")

(def page-files
  (sort (filter #(.endsWith (clojure.string/lower-case %) ".md")
          (file-seq (clojure.java.io/file page-files-path)))))

; connection

(def conn nil)
(def uri "datomic:mem://collardb")

; helper

(defn basename [name] (clojure.string/replace (.getName name) #"\.md$" ""))

(defn timestamp [time]
  (tf/unparse
   (tf/with-zone
     (tf/formatter "yyyy-MM-dd")
     (tc/time-zone-for-id "UTC"))
   (tr/from-long (.getTime (java.util.Date. time)))))

(defn dump-schema []
  (let [db (d/db conn)]
    (clojure.pprint/pprint
     (map #(->> % first (d/entity db) d/touch)
          (d/q '[:find ?v
                 :where [_ :db.install/attribute ?v]]
               db)))))

(defn find-page-id [page-name]
  (ffirst (d/q '[:find ?p
                 :in $ ?n
                 :where [?p :page/name ?n]]
               (d/db conn)
               page-name)))

(defn find-tag-id [tag-name]
  (ffirst (d/q '[:find ?t
                 :in $ ?n
                 :where [?t :tag/name ?n]]
               (d/db conn)
               tag-name)))

; read

(defn find-pages []
  (d/q '[:find ?n
         :where [_ :page/name ?n]]
       (d/db conn)))

(defn find-tags []
  (d/q '[:find ?n
         :where [_ :tag/name ?n]]
       (d/db conn)))

(defn find-page-title [page-name]
  (d/q '[:find ?t
         :in $ ?name
         :where
         [?p :page/name ?n]
         [?p :page/title ?t]]
       (d/db conn)
       page-name))

(defn find-page-tags [page-name]
  (d/q '[:find ?n
         :in $ ?pn
         :where
         [?p :page/name ?pn]
         [?p :page/tags ?t]
         [?t :tag/name ?n]]
       (d/db conn)
       page-name))

; assert

(defn add-page [page-name]
  @(d/transact conn [{:db/id (d/tempid :db.part/user)
                      :page/name page-name}]))

(defn add-title-to-page [page-name title]
  @(d/transact conn [{:db/id (find-page-id page-name)
                      :page/title title}]))

(defn add-text-to-page [page-name text]
  @(d/transact conn [{:db/id (find-page-id page-name)
                      :page/title text}]))

(defn add-time-to-page [page-name time]
  @(d/transact conn [{:db/id (find-page-id page-name)
                      :page/time time}]))

(defn add-tag [tag-name]
  @(d/transact conn [{:db/id (d/tempid :db.part/user)
                      :tag/name tag-name}]))

(defn add-tag-to-page [page-name tag-name]
  (let [tag-id (find-tag-id tag-name)]
    @(d/transact conn [{:db/id (find-page-id page-name)
                        :page/tags tag-id}])))

; db init

(defn fresh-db []
  (d/delete-database uri)
  (d/create-database uri)
  (let [conn (d/connect uri)
        schema (load-file "db/schema.edn")]
    (d/transact conn schema)
    conn))

(defn replenish []
  (doall
   (for [page-file page-files]
     (let [page-name (basename page-file)
           page-lmod (timestamp (.lastModified page-file))
           page-data (md/md-to-html-string-with-meta (slurp page-file) :reference-links? true)
           page-meta (:metadata page-data)
           page-text (:html page-data)
           page-title (if (some? (:title page-meta)) (first (:title page-meta)) page-name)
           page-time (if (some? (:date page-meta)) (timestamp (* (Long/parseLong (first (:date page-meta))) 1000)) nil)
           page-tags (if (some? (:keywords page-meta)) (first (:keywords page-meta)) [])
           ]
       (do
         (add-page page-name)
         (add-title-to-page page-name page-title)
         (add-text-to-page page-name page-text)
         (add-time-to-page page-name page-time)
         (for [tag page-tags]
           (add-tag-to-page page-name tag))
         )))))

(with-redefs [conn (fresh-db)]
  (replenish))
