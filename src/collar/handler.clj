(ns collar.handler
  (:require [collar.views :as views]
            [compojure.core :as cc]
            [compojure.handler :as handler]
            [compojure.route :as route]
            [ring.middleware.defaults :refer [wrap-defaults site-defaults]]))

(cc/defroutes app-routes
  (cc/GET "/"
          []
          (views/page-root))
  (cc/GET "/do"
          []
          (views/page-do))
  (cc/GET "/who"
          []
          (views/page-who))
  (cc/GET "/wip"
          []
          (views/page-wip))
  (route/resources "/")
  (route/not-found "not found"))

(def app
  (wrap-defaults app-routes site-defaults))
