(ns collar.state
  (:require [collar.util :as u]
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
  (let [lmod (.lastModified file)
        name (get-name file)
        old ((keyword name) (:data @state))]
    (if (= (:lmod old) lmod)
      old
      {:lmod lmod
       :data (md/md-to-html-string-with-meta
              (slurp file) :reference-links? true)
       :name name})))

(defn upstate! []
  (let [dir (io/file u/db-path)
        lmod (.lastModified dir)
        old @state]
    (if (= (:lmod old) lmod)
      old
      (let [files (->> dir .listFiles (filter valid?) sort)
            raws (zipmap (map get-name files) (map upstate-f files))]
        (reset! state {:lmod lmod :data raws})))))
