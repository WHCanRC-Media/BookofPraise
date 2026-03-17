melody = \fixed c {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 e'2 b'2 g'2 a'2 a'4 g'4 fis'2 e'2 r2 \break

  % Line 2
  e'2 g'4 a'4 b'4 d''4 cis''4 cis''4 b'2 r2 \break

  % Line 3
  b'2 cis''4 b'4 a'4 g'4 fis'4 e'4 fis'2 e'2 r2 \break

  % Line 4
  g'2 g'4 g'4 a'2 g'4 fis'4 e'2 d'2 r2 \break

  % Line 5
  e'2 b'4 b'4 g'4 b'4 a'2 g'2 fis'2 r2 \break

  % Line 6
  b'2 a'4 g'4 fis'4 e'4 g'4 a'4 fis'2 e'1 \bar "|."
}
