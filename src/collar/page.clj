(ns collar.page
  (:require [collar.piece :as p]
            [collar.state :as s]
            [collar.util :as u]
            [clj-time.coerce :as tr]
            [clj-time.core :as tc]
            [clj-time.format :as tf]
            [clojure.string :as str]
            [hiccup.page :as h]
            [hiccup.util :as hu]))

(def pages-scope "/pages")
(def page-scope "/page")
(def root-scoped ["about" "apps"])

;; template-related

(defn timestamp [secs]
  (if-not (number? secs)
    nil
    (tf/unparse
     (tf/with-zone
       (tf/formatter "yyyy-MM-dd")
       (tc/time-zone-for-id "UTC"))
     (tr/from-long secs))))

(defn get-tag-names [tag-string]
  (-> tag-string (str/split #"\s*,\s*") sort))

(defn create-tag [tagname class]
  [:a {:href (if (= class "clear")
               pages-scope
               (str pages-scope "?tag=" (hu/url-encode tagname)))}
   [:div.tag (if class {:class class}) tagname]])

(defn create-page [pre & text]
  (let [{:keys [short title time lmod tags]} pre]
    (h/html5
     (p/head short)
     (p/nav short)
     [:div.page
      [:div.page-head
       [:h1.title (or title short)]
       (if (seq tags)
         [:div.tags (->> tags
                         (map #(create-tag % "flat"))
                         (interpose ", "))])
       (if (some? time) [:div.time time])
       (if (and (some? lmod) (not= time lmod)) [:div.lmod lmod])]
      [:div.page-body text]])))

;; file-related

(defn root-scoped? [pagename]
  (some #{pagename} root-scoped))

(defn get-page [raw-page]
  (let [{:keys [data name lmod]} raw-page
        {:keys [title date keywords]} (:metadata data)
        short (str/replace name "-" " ")]
    {:name name
     :short short
     :title (if (seq title) (first title) short)
     :tags (if (seq keywords) (-> keywords first get-tag-names) [])
     :time (if (seq date) (-> date first Integer/parseInt (* 1000)))
     :lmod lmod
     :text (:html data)}))

(defn get-pages []
  (:data (s/upstate!)))

;; route-related

(defn flip [pagename]
  (let [which (-> (get-pages) (get pagename))
        page (if which (get-page which))]
    (if page
      (create-page
       {:short (:short page)
        :title (:title page)
        :time (if (:time page) (timestamp (:time page)))
        :lmod (timestamp (:lmod page))
        :tags (:tags page)}
       (:text page))
      (create-page
       {:short "not found"}
       [:span [:em "no page called "] pagename]))))

(def flip-root
  (create-page
   {:short "root"
    :title "welcome"}
   [:p ":-:-)"]))

(defn flip-pages [query-tag]
  (let [all-pages (->> (get-pages) (map peek) (map get-page))
        all-tags (->> all-pages (map :tags) flatten distinct sort)
        pages (if query-tag
                (filter #(some #{query-tag} (:tags %)) all-pages)
                all-pages)
        page-count (count pages)]
    (create-page
     {:short "pages"}
     [:ul.tag-list
      (if query-tag (create-tag "tags" "clear") "tags") ": "
      (for [tag all-tags]
        (create-tag tag (if (= tag query-tag) "active")))]
     [:div.page-count
      page-count " page" (if (not= page-count 1) "s")]
     [:ul.page-list
      {:class (if (= page-count 0) "pageless")}
      (if (= page-count 0)
        [:span [:em "no page tagged "] query-tag]
        (for [page pages]
          (let [{:keys [name, short, tags, lmod]} page
                time (if lmod
                       (timestamp lmod)
                       (timestamp (:time page)))
                link (if (root-scoped? name)
                       name
                       (str page-scope "/" name))]
            [:li
             [:a {:href link}
              [:span.page-name short] " " [:span.time time]]
             (if (seq tags)
               [:div.tags (for [tag tags] (create-tag tag nil))])])))])))
