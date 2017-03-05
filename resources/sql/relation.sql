-- :name create-junction-entry-tag :!
-- :doc Create ENTRY-TAG junction table
create table entry_tag (
  id integer auto_increment primary key,
  id_e integer,
  id_t integer
)

-- :name insert-entry-tag :! :n
-- :doc Insert one ENTRY_TAG relation
insert into entry_tag
(id_e, id_t)
values (:eid, :tid)
