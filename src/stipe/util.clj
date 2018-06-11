(ns stipe.util
  (:require [clj-time.coerce :as tr]
            [clj-time.core :as tc]
            [clj-time.format :as tf]
            [config.core :as c]))

(def env (:env c/env))
(def is-dev? (= env "development"))
(def is-next? (= env "next"))
(def is-master? (= env "master"))

(def site-name
  (cond
    is-dev? (str "localhost:" (:port c/env))
    is-next? "dev.agaric.net"
    is-master? "agaric.net"))
(def db-path "db/")

(defn timestamp [secs]
  (tf/unparse
   (tf/with-zone
     (tf/formatter "yyyy-MM-dd")
     (tc/time-zone-for-id "UTC"))
   (tr/from-long secs)))
