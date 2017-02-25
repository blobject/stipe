(ns collar.core
  (:require [collar.handler :as handler]
            [config.core :refer [env]]
            [ring.adapter.jetty :as jetty]
            [ring.middleware.defaults :as middleware])
  (:gen-class))

(def app
  (middleware/wrap-defaults handler/app-routes middleware/site-defaults))

(defn -main [& [port]]
  (jetty/run-jetty #'app {:port (:port env)
                          :join? false}))
