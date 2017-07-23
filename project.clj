(defproject collar "0.1.0"
  :description "alocy.be base"
  :url "https://alocy.be"
  :min-lein-version "2.0.0"
  :dependencies [[org.clojure/clojure "1.8.0"]
                 [org.clojure/clojurescript "1.9.671"]
                 [compojure "1.6.0"]
                 [yogthos/config "0.8"]
                 [hiccup "1.0.5"]
                 [markdown-clj "0.9.99"]
                 [ring/ring-defaults "0.3.0"]
                 [ring/ring-jetty-adapter "1.6.2"]
                 [clj-time "0.14.0"]]
  :plugins [[lein-figwheel "0.5.11"]
            [lein-ring "0.12.0"]]
  :ring {:handler collar.core/app}
  :main collar.core
  :profiles
  {:dev {:dependencies [[ring/ring-mock "0.3.1"]]
         :resource-paths ["config/development"]}
   :next {:resource-paths ["config/next"]}
   :master {:resource-paths ["config/master"]}
   :uberjar {:aot :all}}
  :jvm-opts ^:replace ["-Xmx1g" "-server"])
