## Error fixing
If this error occur: ` error returned from database: 1044 (42000): Access denied for user 'user'@'localhost' to database 'actix_sqlx' `, 
then follow this procedures to diagnose & fix:

### 1) open termnial, run (MacOS):   
```bash
$ sudo /Applications/XAMPP/xamppfiles/bin/mysql -u root -p
```
( this will connect to mysql as username: "root" - the user with greatest permissions )
( if success, you'll see sth like this: "Welcome to MariaDB monitor." )
Note:  The password prompt of Unix (MacOS): "Password: "
       The password prompt of MySQL login: "Enter password: "
       => So don't mess them 
       
### 2) Run: 
```bash
$ show databases;
```

As your logged-in user now is root, you should be able to see all database like in phpmyadmin.
* eg. 
```
+--------------------+
| Database           |
+--------------------+
| actix_sqlx         |
| information_schema |
| mysql              |
| performance_schema |
| phpmyadmin         |
| test               |
+--------------------+
6 rows in set (0.001 sec)
```

### 3) press Ctrl + C to log out from root.

### 4) Now try the user with username `user` we created to see if it has enough permissions/
* Run:   
```bash
$ sudo /Applications/XAMPP/xamppfiles/bin/mysql -u user -p
```
* then run:
```bash
$ show databases;
```

If you only see very few databases like this, it means the user `user` does not have permissions.
* eg.
```
+--------------------+
| Database           |
+--------------------+
| information_schema |
| test               |
+--------------------+
2 rows in set (0.001 sec)
```

### 5) If you're in the case as mentioned in 4), do the followings:
* Login as root for full permission:  
```bash
$ sudo /Applications/XAMPP/xamppfiles/bin/mysql -u root -p
```

Or you can use phpmyadmin if you has access to it.
* Run this query command:
```bash
use mysql;
SET PASSWORD FOR 'user'@'localhost' = PASSWORD('password');
GRANT ALL PRIVILEGES ON *.* TO 'user'@'localhost';
FLUSH PRIVILEGES;
```
This will reset the password of the user `'user'`@`'localhost'` to `'password'`,
grant all permissions to the user then refresh.

### 6) press Ctrl + C to log out in terminal,
* then login again:
```bash
$ sudo /Applications/XAMPP/xamppfiles/bin/mysql -u user -p
```
* Then run:
```bash
$ show databases;
```
Now user `user` has already been granted the highest permissions, you should be able to see all databases:
* eg.
```
+--------------------+
| Database           |
+--------------------+
| actix_sqlx         |
| information_schema |
| mysql              |
| performance_schema |
| phpmyadmin         |
| test               |
+--------------------+
6 rows in set (0.001 sec)
```
Now use the following connection string to connect: `mysql://user:password@127.0.0.1:3306/actix_sqlx`