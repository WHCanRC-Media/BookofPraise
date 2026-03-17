melody = \fixed c {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 b'2 c''2 b'4 g'4 a'4 g'4 fis'2 e'2 r2 \break

  % Line 2
  e'2 g'2 fis'4 e'4 e'4 dis'4 e'2 r2 \break

  % Line 3
  g'2 e'2 g'4 a'4 b'4 g'4 c''2 b'2 r2 \break

  % Line 4
  b'2 c''2 b'4 a'4 a'4 gis'4 a'2 r2 \break

  % Line 5
  a'2 b'2 g'4 e'4 g'4 fis'4 e'2 r2 \break

  % Line 6
  e'2 g'2( e'4) a'4 a'4 gis'4 a'1 \bar "|."
}
