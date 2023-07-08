begin;
select plan(11); -- only one statement to run

SELECT has_table('profiles' );
SELECT col_is_pk('profiles', 'id' );
SELECT has_column('profiles','user_id');

SELECT policies_are(
  'profiles',
  ARRAY [
    'View all profiles',
    'Update own profile'
  ]
);

INSERT INTO auth.users (instance_id,id,aud,"role",email,encrypted_password,email_confirmed_at,last_sign_in_at,raw_app_meta_data,raw_user_meta_data,is_super_admin,created_at,updated_at,phone,phone_confirmed_at,confirmation_token,email_change,email_change_token_new,recovery_token) VALUES
	('00000000-0000-0000-0000-000000000000'::uuid,'f76629c5-a070-4bbc-9918-64beaea48848'::uuid,'authenticated','authenticated','test@example.com','$2a$10$PznXR5VSgzjnAp7T/X7PCu6vtlgzdFt1zIr41IqP0CmVHQtShiXxS','2022-02-11 21:02:04.547','2022-02-11 22:53:12.520','{"provider": "email", "providers": ["email"]}','{}',FALSE,'2022-02-11 21:02:04.542','2022-02-11 21:02:04.542',NULL,NULL,'','','',''),
	('00000000-0000-0000-0000-000000000000'::uuid,'d9064bb5-1501-4ec9-bfee-21ab74d645b8'::uuid,'authenticated','authenticated','demo@example.com','$2a$10$mOJUAphJbZR4CdM38.bgOeyySurPeFHoH/T1s7HuGdpRb7JgatF7K','2022-02-12 07:40:23.616','2022-02-12 07:40:23.621','{"provider": "email", "providers": ["email"]}','{}',FALSE,'2022-02-12 07:40:23.612','2022-02-12 07:40:23.613',NULL,NULL,'','','','')
ON CONFLICT (id) DO NOTHING;

SELECT results_eq(
               $$ SELECT email as text FROM auth.users ORDER BY email DESC $$,
               ARRAY [ 'test@example.com'::varchar(255), 'demo@example.com'::varchar(255) ]
           );
DECLARE count_profiles_by_id CURSOR FOR SELECT count(id) FROM public.profiles;
DECLARE count_users_by_id CURSOR FOR SELECT count(id) FROM auth.users;
SELECT results_eq('count_profiles_by_id'::refcursor,ARRAY [2::bigint]);
SELECT results_eq('count_users_by_id'::refcursor,ARRAY [2::bigint]);
CLOSE count_profiles_by_id;
CLOSE count_users_by_id;

DECLARE profiles_userids CURSOR FOR SELECT user_id FROM public.profiles ORDER BY user_id;
DECLARE auth_userids CURSOR FOR SELECT id FROM auth.users ORDER BY id;
SELECT results_eq('profiles_userids'::refcursor,'auth_userids'::refcursor);
CLOSE profiles_userids;
CLOSE auth_userids;


DELETE FROM auth.users WHERE email = 'test@example.com';

DECLARE count_profiles_by_id CURSOR FOR SELECT count(id) FROM public.profiles;
DECLARE count_users_by_id CURSOR FOR SELECT count(id) FROM auth.users;
SELECT results_eq('count_profiles_by_id'::refcursor,ARRAY [2::bigint]);
SELECT results_eq('count_users_by_id'::refcursor,ARRAY [1::bigint]);
CLOSE count_profiles_by_id;
CLOSE count_users_by_id;

DECLARE profiles_userids CURSOR FOR SELECT user_id FROM public.profiles WHERE user_id IS NOT NULL ORDER BY user_id;
DECLARE auth_userids CURSOR FOR SELECT id FROM auth.users ORDER BY id;
SELECT results_eq('profiles_userids'::refcursor,'auth_userids'::refcursor);
CLOSE profiles_userids;
CLOSE auth_userids;

select * from finish();
rollback;