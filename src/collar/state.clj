(ns collar.state
  (:require [collar.util :as u]))

(defonce state (atom {}))

; Currently, only the "last-modified time" of the db folder is saved.
; This is used, for example, to determine whether to re-read files in db.
(defn state-changed?
  "Whether global state has changed."
  []
  (let [db-mod (.lastModified (clojure.java.io/file u/db-path))]
    (if (not= (:db-mod @state) db-mod)
      (do
        (swap! state assoc :db-mod db-mod)
        true)
      false)))

