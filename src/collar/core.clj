(ns collar.core
  (:require [collar.route :as r]
            [aleph.http :as a]
            [aleph.netty :as an]
            [config.core :as c]
            [ring.middleware.defaults :as m])
  (:gen-class))

(def app
  (m/wrap-defaults
   r/routes
   m/site-defaults))

(defn -main [& [port]]
  (an/wait-for-close
   (a/start-server #'app {:port (:port c/env)})))
