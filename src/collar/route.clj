(ns collar.route
  (:require [collar.view :as view]
            [compojure.core :as compojure]
            [compojure.route :as route]))

(compojure/defroutes routes
  (compojure/GET "/" [] view/root)
  (compojure/GET "/new" [] view/new)
  (compojure/GET "/pages" [] view/pages)
  (compojure/GET "/tags" [] view/tags)
  (compojure/GET "/who" [] view/who)
  (route/resources "/")
  (route/not-found "not found"))
