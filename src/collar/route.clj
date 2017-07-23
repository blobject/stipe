(ns collar.route
  (:require [collar.page :as page]
            [compojure.core :as compojure]
            [compojure.route :as route]))

(compojure/defroutes routes
  (compojure/GET "/" []
                 page/root)
  (compojure/GET "/about" []
                 (page/flip "about"))
  (compojure/GET "/pages" []
                 page/pages)
  (compojure/GET "/page/:p" [p]
                 (page/flip (clojure.string/lower-case p)))
  (route/resources "/")
  (route/not-found "not found"))
