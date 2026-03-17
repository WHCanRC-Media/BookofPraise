melody = \fixed c {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  b'2 a'4 a'4 b'2 d''2 cis''4 cis''4 b'2 r2 \break

  % Line 2
  a'2 b'4 cis''4 d''4 cis''4 b'2 b'2 a'2 r2 \break

  % Line 3
  b'2 d''4 cis''4 b'4 a'4 fis'4 a'4 g'2 fis'2 r2 \break

  % Line 4
  d'2 fis'4 g'4 a'4 fis'4 g'4 a'4 b'2 a'2 r2 \break

  % Line 5
  fis'2 a'4 a'4 b'4 a'4 d''4 d''4 cis''2 b'2 r2 \break

  % Line 6
  b'2 a'4 b'4 d''2 cis''2 b'4 a'4 g'2 fis'1 \bar "|."
}
