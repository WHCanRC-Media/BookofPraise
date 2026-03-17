melody = \relative c'' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g2 fis4 g4 a4 b2 a2 gis4 a2 r2 \break

  % Line 2
  a2 b2 c2 a4 a4 b4 c4 d2 r2 \break

  % Line 3
  d2 c4 c4 d4 a4 c2 b2 a2 r2 \break

  % Line 4
  a2 g4 a4 e4 g4 fis2 e2 d2 r2 \break

  % Line 5
  d2 a'4 a4 e4 fis4 g4 a4 fis2 e2 r2 \break

  % Line 6
  d2 a'4 a4 e4 fis4 g4 a4 fis2 e2 r2 a2 b4 b4 g4 g4 c2 b2 a2 r2 \break

  % Line 7
  a2 a4 b4 g4 a4 fis2 e2 d1 \bar "|."
}
