(ns collar.route
  (:require [collar.page :as p]
            [compojure.core :as c]
            [compojure.route :as r]))

(c/defroutes routes
  (c/GET "/" []
         p/root-page)
  (c/GET "/about" []
         (p/flip "about"))
  (c/GET "/pages" [tag]
         (p/pages-page tag))
  (c/GET "/page/:p" [p]
         (p/flip (clojure.string/lower-case p)))
  (r/resources "/")
  (r/not-found "not found"))
