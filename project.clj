(defproject stipe "0.1.0"
  :description "agaric.net base"
  :url "https://agaric.net"
  :min-lein-version "2.0.0"
  :dependencies [[aleph "0.4.4"]
                 [clj-time "0.14.2"]
                 [org.clojure/clojure "1.9.0"]
                 [org.clojure/clojurescript "1.9.946"]
                 [compojure "1.6.0"]
                 [yogthos/config "0.9"]
                 [hiccup "1.0.5"]
                 [markdown-clj "1.0.1"]
                 [ring/ring-defaults "0.3.1"]]
  :plugins [[lein-ancient "0.6.14"]
            [lein-figwheel "0.5.14"]
            [lein-ring "0.12.2"]]
  :ring {:handler stipe.core/app}
  :main stipe.core
  :profiles
  {:dev {:dependencies [[ring/ring-mock "0.3.2"]]
         :resource-paths ["config/development"]}
   :next {:resource-paths ["config/next"]}
   :master {:resource-paths ["config/master"]}
   :uberjar {:aot :all}}
  :jvm-opts ^:replace ["-Xmx1g" "-server"])
