(ns collar.view.piece
  (:require [clj-time.coerce :as tc]
            [clj-time.format :as tf]))

(defn tag [tag]
  [:div.tag
   [:div.name (:name tag)]])

(defn clip [entry]
  [:a.clip {:href (str "/" (:title entry))}
   [:div.title (:title entry)]
   [:div.time (tf/unparse
               (tf/formatter "yyyy-MM-dd")
               (tc/from-long (.getTime (:time_create entry))))]])
