(ns collar.util
  (:require [clj-time.coerce :as tr]
            [clj-time.core :as tc]
            [clj-time.format :as tf]
            [config.core :as c]))

(def site-name "alocy.be")
(def site-path (str "https://" site-name))
(def db-path "db/")

(def env (:env c/env))
(def is-dev? (= env "development"))
(def is-next? (= env "next"))
(def is-master? (= env "master"))

(defn timestamp [secs]
  (tf/unparse
   (tf/with-zone
     (tf/formatter "yyyy-MM-dd")
     (tc/time-zone-for-id "UTC"))
   (tr/from-long secs)))
