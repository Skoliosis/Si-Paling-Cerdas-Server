CREATE TABLE IF NOT EXISTS QuestionLists (
  ID INT NOT NULL PRIMARY KEY AUTO_INCREMENT,
  Question VARCHAR(1024) NOT NULL,
  AnswerOption1 VARCHAR(1024) NOT NULL,
  AnswerOption2 VARCHAR(1024) NOT NULL,
  AnswerOption3 VARCHAR(1024) NOT NULL,
  AnswerOption4 VARCHAR(1024) NOT NULL,
  AnswerIndex INT NOT NULL
);

-- IF (EXISTS (SELECT * FROM information_schema.tables 
--             WHERE table_schema = 'si_paling_cerdas' AND table_name = 'QuestionLists' LIMIT 1))
-- BEGIN
    INSERT INTO QuestionLists 
        (Question, AnswerOption1, AnswerOption2, AnswerOption3, AnswerOption4, AnswerIndex)
    VALUES
        ('Berapa Jumlah Provinsi di Indonesia?', '36', '37', '38', '39', 2),
        ('Negara Terkecil di Dunia adalah?', 'Vatikan', 'Chili', 'Bolivia', 'Hawai', 0),
        ('Berapa Jumlah Tulang Rusuk Manusia?', '24', '22', '26', '20', 0),
        ('Warna yang Paling Panjang dalam Pelangi adalah Warna?', 'Merah', 'Nila', 'Ungu', 'Kuning', 0),
        ('Siapa pencipta lagu Indonesia Raya?', 'R.Kusbini', 'Ismail Marzuki', 'W.R Supratman', 'Ibu Soed', 2),
        ('Apa saja Warna Primer?', 'Biru, Merah, Hijau', 'Merah, Kuning, Biru', 'Kuning, Biru, Hitam', 'Putih, Hijau, Hitam', 1),
        ('Apa nama kerajaan yang pertama kali berdiri di Indonesia?', 'Kutai', 'Sriwijaya', 'Majapahit', 'Singasari', 0),
        ('“Starry Night” adalah Karya Lukisan dari?', 'Leonardo da Vinci', 'Pablo Picasso', 'Claude Monet', 'Vincent van Gogh', 3),
        ('Apa Benua Terbesar di Dunia?', 'Benua Eropa', 'Benua Asia', 'Benua Amerika', 'Benua Antartika', 1),
        ('Jumlah Benua di Dunia', '6', '5', '8', '7', 3),
        ('Negara manakah yang memiliki wilayah terluas di dunia?', 'China', 'Rusia', 'Amerika Serikat', 'Vietnam', 1),
        ('Harry Potter adalah Novel yang Ditulis Oleh?', 'George Orwell', 'Raditya Dika', 'J.K Rowling', 'Rick Riordan', 2),
        ('Ada berapa negara yang tergabung dalam ASEAN?', '9', '12', '10', '11', 3),
        ('Apa Ibu Kota Jawa Tengah', 'Surakarta', 'Solo', 'Semarang', 'Surabaya', 2),
        ('Apa nama mata uang Thailand?', 'Rupiah', 'Bath', 'Euro', 'Dolar', 1),
        ('5 + 3 * 4', '32', '17', '18', '33', 1),
        ('Pada tahun berapa Jepang menyerang Pearl Harbor, mengakibatkan Amerika Serikat ikut terlibat dalam Perang Dunia II?', '1941', '1942', '1943', '1944', 0),
        ('Jika 2x + 1 = 5, berapakah nilai dari x ?', '1', '2', '3', '4', 1),
        ('Jika x + y = 9 dan x - y = -5, berapakah nilai dari 2x + y ?', '9', '10', '11', '20', 2),
        ('Washington D.C Ibukota Amerika Serikat Terletak di Benua?', 'Amerika Timur', 'Amerika Barat', 'Amerika Selatan', 'Amerika Utara', 3);
-- END
