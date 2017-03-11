(ns collar.route
  (:require [collar.view :as view]
            [compojure.core :as compojure]
            [compojure.route :as route]))

(compojure/defroutes routes
  (compojure/GET "/" [] view/root)
  (compojure/GET "/draft" [] view/draft)
  (compojure/GET "/pages" [] view/pages)
  (compojure/GET "/who" [] view/who)
  (route/resources "/")
  (route/not-found "not found"))
