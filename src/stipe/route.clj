(ns stipe.route
  (:require [stipe.page :as p]
            [compojure.core :as c]
            [compojure.route :as r]))

(c/defroutes routes
  (c/GET "/" []
         p/flip-root)
  (c/GET "/about" []
         (p/flip "about"))
  (c/GET "/dev" []
         (p/flip "dev"))
  (c/GET "/dev/" []
         (p/flip "dev"))
  (c/GET "/pages" [tag]
         (p/flip-pages tag))
  (c/GET "/pages/" [tag]
         (p/flip-pages tag))
  (c/GET "/page/:p" [p]
         (p/flip (clojure.string/lower-case p)))
  (r/resources "/")
  (r/not-found (p/flip nil)))
