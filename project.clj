(defproject collar "0.1.0"
  :description "alocy.be base"
  :url "http://alocy.be"
  :min-lein-version "2.0.0"
  :dependencies [[org.clojure/clojure "1.8.0"]
                 [org.clojure/clojurescript "1.9.671"]
                 [compojure "1.5.1"]
                 [yogthos/config "0.8"]
                 [com.datomic/datomic-free "0.9.5561.50"]
                 [expectations "2.2.0-beta1"]
                 [hiccup "1.0.5"]
                 [markdown-clj "0.9.99"]
                 [ring/ring-defaults "0.2.1"]
                 [ring/ring-jetty-adapter "1.5.0"]
                 [clj-time "0.13.0"]]
  :plugins [[lein-autoexpect "1.9.0"]
            [lein-figwheel "0.5.11"]
            [lein-ring "0.9.7"]]
  :ring {:handler collar.core/app}
  :main collar.core
  :profiles
  {:dev {:dependencies [[javax.servlet/servlet-api "2.5"]
                        [ring/ring-mock "0.3.0"]]
         :resource-paths ["config/development"]}
   :next {:resource-paths ["config/next"]}
   :master {:resource-paths ["config/master"]}
   :uberjar {:aot :all}}
  :jvm-opts ^:replace ["-Xmx1g" "-server"])
