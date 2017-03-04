(ns collar.view.piece)

(defn tag [tag]
  [:div.tag
   [:div.name (:name tag)]
   [:div.count (int (:count tag))]])

(defn clip [entry]
  [:div.clip
   [:div.title (:title entry)]
   [:div.time (:time entry)]
   [:div.tags (map #(tag (str %)) (:tags entry))]])
