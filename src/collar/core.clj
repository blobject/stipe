(ns collar.core
  (:require [collar.route :as r]
            [config.core :as c]
            [ring.adapter.jetty :as j]
            [ring.middleware.defaults :as m])
  (:gen-class))

(def app
  (m/wrap-defaults
   r/routes
   m/site-defaults))

(defn -main [& [port]]
  (j/run-jetty #'app {:port (:port c/env)
                      :join? false}))
