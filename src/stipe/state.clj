(ns stipe.state
  (:require [stipe.util :as u]
            [clojure.java.io :as io]
            [clojure.string :as s]
            [markdown.core :as md]))

(defonce state (atom {}))

(defn valid? [file]
  (and
   (-> file .getName s/lower-case (.endsWith ".md"))
   (> (.length file) 0)))

(defn get-name [file]
  (-> file .getName (s/replace #"\.md$" "")))

(defn upstate-f [file]
  (let [name (get-name file)
        lmod (.lastModified file)
        old (get @state name)
        f (fn []
            (let [data (md/md-to-html-string-with-meta
                        (slurp file) :reference-links? true)]
              {:lmod lmod :data data :name name}))]
    (if (= (:lmod old) lmod) old (f))))

(defn upstate []
  (let [files (->> (io/file u/db-path) .listFiles (filter valid?) sort)
        new (zipmap (map get-name files) (map upstate-f files))]
    (reset! state new)))
