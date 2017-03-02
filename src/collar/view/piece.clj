(ns collar.view.piece)

(defn tag [tag]
  [:div.tag
   [:div.name (:name tag)]
   [:div.count (int (:count tag))]])

(defn clip [entry]
  [:div.clip])
