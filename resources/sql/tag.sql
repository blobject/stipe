-- :name create-table-tag :!
-- :doc Create TAG table
create table tag (
  id          integer auto_increment primary key,
  name        varchar,
  time_create timestamp not null default current_timestamp
)

-- :name insert-tag :! :n
-- :doc Insert one TAG
insert into tag
(name)
values (:name)

-- :name get-tags :? :*
-- :doc Select all TAG rows
select * from tag
