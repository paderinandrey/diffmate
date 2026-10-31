INSERT INTO targets VALUES (1, 'o', 'r', 1, 'untracked', 100);
INSERT INTO jobs VALUES (1, 'review', 1, 'succeeded', 4242, 90, NULL, 100, 200);
INSERT INTO runs VALUES (
  1, 1, 1, 'succeeded', 'findings', NULL, NULL, NULL,
  '1111111111111111111111111111111111111111',
  '2222222222222222222222222222222222222222',
  '3333333333333333333333333333333333333333',
  'main', 4243, 4243, 0.125, 80, '["warning"]', 120, 200
);
INSERT INTO drafts VALUES (1, 1, '{"original":"output"}', 'limitation', '["improvement"]', 200);
INSERT INTO findings VALUES (
  '1.1', 1, 1, 1, 'high', 'src/lib.rs', 3, 'RIGHT', 'Title', 'Body', NULL, 'undecided'
);
