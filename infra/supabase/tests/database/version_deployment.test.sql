begin;
select plan(12); -- only one statement to run

SELECT has_table('version_deployments' );

-- Check admin can insert
SELECT lives_ok($$ INSERT INTO public.version_deployments (version) VALUES ('0.0.1'); $$);

-- Check only 1 version can be scheduled,running,deployed,finished at a time
SELECT throws_like($$ INSERT INTO public.version_deployments (version) VALUES ('0.0.1'); $$,
    'Version 0.0.1 is already in non-failed status');

-- Check semantic versioning is enforced
SELECT throws_like($$ INSERT INTO public.version_deployments (version) VALUES ('toto'); $$,
    '%new row for relation "version_deployments" violates check constraint "version_deployments_version_check"%');

-- Check admin can read
DECLARE list_version_deployments CURSOR FOR SELECT version, status FROM public.version_deployments;
SELECT results_eq('list_version_deployments'::refcursor, $$VALUES ('0.0.1'::varchar(20), 'scheduled'::version_deployment_status)$$);
CLOSE list_version_deployments;

-- Check admin can update the status
SELECT lives_ok($$ UPDATE public.version_deployments SET status = 'deployed' WHERE version = '0.0.1'; $$);
SELECT results_eq('SELECT status FROM public.version_deployments WHERE version = ''0.0.1''',ARRAY ['deployed'::version_deployment_status]);

-- Check Only scheduled status is authorized for inserted rows
SELECT throws_like($$ INSERT INTO public.version_deployments (version, status) VALUES ('0.0.2', 'deployed'); $$,
    'Only "scheduled" status is authorized when inserting.');

-- Check only 1 version can be deployed at a time
INSERT INTO public.version_deployments (version) VALUES ('0.0.2');
SELECT throws_like($$ UPDATE public.version_deployments SET status = 'deployed' WHERE version = '0.0.2'; $$,
    'Only one version can be set to completed.');

--- Check with a user ---
INSERT INTO auth.users (instance_id,id,aud,"role",email,encrypted_password,email_confirmed_at,last_sign_in_at,raw_app_meta_data,raw_user_meta_data,is_super_admin,created_at,updated_at,phone,phone_confirmed_at,confirmation_token,email_change,email_change_token_new,recovery_token) VALUES
	('00000000-0000-0000-0000-000000000000'::uuid,'d9064bb5-1501-4ec9-bfee-21ab74d645b8'::uuid,'authenticated','authenticated','demo@example.com','$2a$10$mOJUAphJbZR4CdM38.bgOeyySurPeFHoH/T1s7HuGdpRb7JgatF7K','2022-02-12 07:40:23.616','2022-02-12 07:40:23.621','{"provider": "email", "providers": ["email"]}','{}',FALSE,'2022-02-12 07:40:23.612','2022-02-12 07:40:23.613',NULL,NULL,'','','','')
ON CONFLICT (id) DO NOTHING;

CALL auth.login_as_user('demo@example.com');

-- Check user can not insert
SELECT throws_like( $$ INSERT INTO public.version_deployments (version) VALUES ('0.0.3'); $$,
        '%new row violates row-level security policy for table "version_deployments"');
-- Check user can not update
UPDATE public.version_deployments SET status = 'finished' WHERE version = '0.0.1';
SELECT results_eq('SELECT status FROM public.version_deployments WHERE version = ''0.0.1''',ARRAY ['deployed'::version_deployment_status]);

-- Check user can read
DECLARE list_version_deployments CURSOR FOR SELECT version FROM public.version_deployments WHERE status = 'deployed';
SELECT results_eq('list_version_deployments'::refcursor,ARRAY ['0.0.1'::varchar(20)]);
CLOSE list_version_deployments;

CALL auth.logout();

SELECT * FROM finish();
ROLLBACK;