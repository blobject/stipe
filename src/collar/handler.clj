(ns collar.handler
  (:require [collar.view :as view]
            [compojure.core :as cc]
            [compojure.handler :as handler]
            [compojure.route :as route]))

(cc/defroutes app-routes
  (cc/GET "/"
          []
          (view/page-root))
  (cc/GET "/do"
          []
          (view/page-do))
  (cc/GET "/who"
          []
          (view/page-who))
  (cc/GET "/wip"
          []
          (view/page-wip))
  (route/resources "/")
  (route/not-found "not found"))
