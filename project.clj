(defproject stipe "0.1.0"
  :description "b.agaric.net base"
  :url "https://b.agaric.net"
  :min-lein-version "2.0.0"
  :dependencies [[aleph "0.4.6"]
                 [clj-time "0.14.4"]
                 [org.clojure/clojure "1.9.0"]
                 [org.clojure/clojurescript "1.10.339"]
                 [compojure "1.6.1"]
                 [yogthos/config "1.1.1"]
                 [hiccup "1.0.5"]
                 [markdown-clj "1.0.2"]
                 [ring/ring-defaults "0.3.2"]]
  :plugins [[lein-ancient "0.6.15"]
            [lein-figwheel "0.5.16"]
            [lein-ring "0.12.4"]]
  :ring {:handler stipe.core/app}
  :main stipe.core
  :profiles
  {:dev {:dependencies [[ring/ring-mock "0.3.2"]]
         :resource-paths ["config/development"]}
   :next {:resource-paths ["config/next"]}
   :master {:resource-paths ["config/master"]}
   :uberjar {:aot :all}}
  :jvm-opts ^:replace ["-Xmx1g" "-server"])
