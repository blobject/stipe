(ns collar.db
  (:require [config.core :refer [env]]
            [monger.collection :as mc]
            [monger.conversion :as mv]
            [monger.core :as m]))

(def base (m/connect-via-uri (:db env)))

(defn get-coll [coll]
  (let [db (:db base)]
    (mc/find-maps db coll)))
