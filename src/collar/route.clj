(ns collar.route
  (:require [collar.page :as page]
            [compojure.core :as compojure]
            [compojure.route :as route]))

(def page-names
  (map #(clojure.string/replace % #"^.*/(.*)\.md$" "$1")
       page/page-paths))

(compojure/defroutes routes
  (compojure/GET "/" [] page/root)
  (compojure/GET "/pages" [] page/pages)
  (compojure/GET "/:pg" [pg]
                 (let [p (clojure.string/lower-case pg)]
                   (if (some #{p} page-names)
                     (page/flip p)
                     (str "\"" pg "\" not found"))))
  (route/resources "/")
  (route/not-found "not found"))
