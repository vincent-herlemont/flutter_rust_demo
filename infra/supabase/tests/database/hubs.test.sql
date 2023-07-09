begin;
select plan(2); -- only one statement to run

SELECT has_table('hubs' );
SELECT has_table('hub_instances' );

select * from finish();
rollback;