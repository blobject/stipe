(ns collar.route
  (:require [collar.page :as page]
            [compojure.core :as compojure]
            [compojure.route :as route]))

(def page-names (map page/basename page/page-files))

(compojure/defroutes routes
  (compojure/GET "/" [] page/root)
  (compojure/GET "/about" [] (page/flip "about"))
  (compojure/GET "/pages" [] page/pages)
  (compojure/GET "/page/:pg" [pg]
                 (let [p (clojure.string/lower-case pg)]
                   (if (and (some #{p} page-names)
                            (not (some #{p} page/specials)))
                     (page/flip p)
                     (str "\"" pg "\" not found"))))
  (route/resources "/")
  (route/not-found "not found"))
