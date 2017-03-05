-- resources/sql/db/sql

-- :name create-table-entry :!
-- :doc Create ENTRY table
create table entry (
  id          integer auto_increment primary key,
  title       varchar,
  body        varchar,
  time_create timestamp not null default current_timestamp
)

-- :name insert-entry :! :n
-- :doc Insert one ENTRY
insert into entry
(title, body)
values (:title, :body)

-- :name get-entries :? :*
-- :doc Select all ENTRY rows
select * from entry

-- :name get-entry-by-title :? :1
-- :doc Select one ENTRY row by :title
select * from entry
where upper(title) = upper(:title)
