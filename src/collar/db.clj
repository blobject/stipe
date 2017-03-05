(ns collar.db
  (:require [config.core :refer [env]]
            [hugsql.core :as h]))

(def spec (:db-uri env))

(h/def-db-fns "sql/entry.sql")
(h/def-db-fns "sql/tag.sql")
(h/def-db-fns "sql/relation.sql")

;(def pool-spec
;  (let [partitions 3
;        ds (doto (BoneCPDataSource.)
;             (.setJdbcUrl (:db env))
;             (.setMinConnectionsPerPartition 5)
;             (.setMaxConnectionsPerPartition 10)
;             (.setPartitionCount partitions)
;             (.setStatisticsEnabled true)
;             (.setIdleConnectionTestPeriodInMinutes 30)
;             (.setIdleMaxAgeInMinutes (* 3 60))
;             (.setConnectionTestStatement "select 1"))]
;    {:datasource ds}))
;
;(def pool (delay pool-spec))
;
;(defn conn [] @pool)
